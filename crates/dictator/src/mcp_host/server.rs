//! Main MCP server using mcp-host framework

use anyhow::Result;
use mcp_host::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::mcp::regime::run_stalint_check;
use crate::mcp::state::{
    CONFIG_FILE, STALINT_CHECK_TIMEOUT_SECS, ServerState, WATCHER_CHECK_INTERVAL_SECS,
};
use crate::mcp::utils::log_to_file;

use super::prompts::DictatorPrompts;
use super::resources::DictatorResources;
use super::tools::{DictatorTools, OccupyTool, StalintWatchTool};

/// Run the MCP server with mcp-host framework
pub fn run() -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run_async())
}

async fn run_async() -> Result<()> {
    log_to_file("=== Dictator MCP server starting (mcp-host) ===");
    log_to_file(&format!("Version: {}", env!("CARGO_PKG_VERSION")));
    log_to_file(&format!("PID: {}", std::process::id()));

    // Create server
    let server = Server::new("dictator", env!("CARGO_PKG_VERSION"));

    // Set concise server instructions for LLMs
    server.set_instructions(Some(
        "Run stalint before any commit. User expects disciplined agents.".to_string(),
    ));

    // Get notification sender for background tasks
    let notification_tx = server.notification_sender();

    // Create shared state with proper notification channel
    let watcher_state = Arc::new(Mutex::new(ServerState::new(notification_tx.clone())));

    // Register macro-based tools (stalint, dictator)
    let tools = Arc::new(DictatorTools {
        state: Arc::clone(&watcher_state),
    });
    DictatorTools::router().register_all(
        server.tool_registry(),
        server.prompt_manager(),
        server.resource_manager(),
        tools,
    );

    // Register stateful tools (manual impl - need notification_tx)
    server.tool_registry().register(StalintWatchTool {
        state: Arc::clone(&watcher_state),
        notification_tx: notification_tx.clone(),
    });
    server.tool_registry().register(OccupyTool {
        state: Arc::clone(&watcher_state),
        notification_tx: notification_tx.clone(),
    });

    // Register macro-based resources
    let resources = Arc::new(DictatorResources {
        state: Arc::clone(&watcher_state),
    });
    DictatorResources::router().register_all(
        server.tool_registry(),
        server.prompt_manager(),
        server.resource_manager(),
        resources,
    );

    // Register macro-based prompts
    let prompts = Arc::new(DictatorPrompts);
    DictatorPrompts::router().register_all(
        server.tool_registry(),
        server.prompt_manager(),
        server.resource_manager(),
        prompts,
    );

    // Update capabilities
    let caps = ServerCapabilities {
        tools: Some(mcp_host::protocol::capabilities::ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(mcp_host::protocol::capabilities::ResourcesCapability {
            subscribe: Some(false),
            list_changed: Some(true),
            list_templates: Some(false),
        }),
        prompts: Some(mcp_host::protocol::capabilities::PromptsCapability {
            list_changed: Some(true),
        }),
        ..Default::default()
    };
    server.set_capabilities(caps);

    // Start background tasks
    start_config_watcher(Arc::clone(&watcher_state), notification_tx.clone());
    start_watcher_check_loop(Arc::clone(&watcher_state), notification_tx);

    // Run server with stdio transport
    let transport = StdioTransport::new();
    server
        .run(transport)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(())
}

/// Background loop that watches .dictate.toml for changes
fn start_config_watcher(
    state: Arc<Mutex<ServerState>>,
    notif_tx: mpsc::UnboundedSender<JsonRpcNotification>,
) {
    tokio::spawn(async move {
        use notify::{RecursiveMode, Watcher};

        let Ok(cwd) = std::env::current_dir() else {
            return;
        };
        let config_path = cwd.join(CONFIG_FILE);

        // Set up file watcher
        let state_clone = Arc::clone(&state);
        let watcher =
            notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res
                    && (event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove())
                    && let Ok(mut s) = state_clone.lock()
                {
                    s.config_dirty = true;
                }
            });

        let mut watcher = match watcher {
            Ok(w) => w,
            Err(e) => {
                log_to_file(&format!("Failed to create config watcher: {e}"));
                return;
            }
        };

        let watch_path = if config_path.exists() {
            config_path.clone()
        } else {
            cwd.clone()
        };

        if let Err(e) = watcher.watch(&watch_path, RecursiveMode::NonRecursive) {
            log_to_file(&format!("Failed to watch config: {e}"));
            return;
        }

        log_to_file(&format!("Watching config: {}", watch_path.display()));

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let is_dirty = {
                let s = state.lock().unwrap();
                s.config_dirty
            };

            if is_dirty {
                {
                    let mut s = state.lock().unwrap();
                    s.config_dirty = false;
                    s.reload_config();
                }

                // Send tools/list_changed notification
                let tools_notification = JsonRpcNotification {
                    jsonrpc: "2.0".to_string(),
                    method: "notifications/tools/list_changed".to_string(),
                    params: Some(serde_json::json!({})),
                };
                let _ = notif_tx.send(tools_notification);

                // Send resources/list_changed notification
                let resources_notification = JsonRpcNotification {
                    jsonrpc: "2.0".to_string(),
                    method: "notifications/resources/list_changed".to_string(),
                    params: Some(serde_json::json!({})),
                };
                let _ = notif_tx.send(resources_notification);

                // Send prompts/list_changed notification
                let prompts_notification = JsonRpcNotification {
                    jsonrpc: "2.0".to_string(),
                    method: "notifications/prompts/list_changed".to_string(),
                    params: Some(serde_json::json!({})),
                };
                let _ = notif_tx.send(prompts_notification);

                log_to_file("Config changed: sent list_changed for tools/resources/prompts");
            }
        }
    });
}

/// Background loop that checks watched paths
fn start_watcher_check_loop(
    state: Arc<Mutex<ServerState>>,
    notif_tx: mpsc::UnboundedSender<JsonRpcNotification>,
) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(WATCHER_CHECK_INTERVAL_SECS)).await;

            let should_lint = {
                let state = state.lock().unwrap();
                state.dirty
                    && state.last_check.elapsed() >= Duration::from_secs(STALINT_CHECK_TIMEOUT_SECS)
            };

            if should_lint {
                let paths: Vec<String> = {
                    let mut state = state.lock().unwrap();
                    state.dirty = false;
                    state.last_check = std::time::Instant::now();
                    state.paths.iter().cloned().collect()
                };

                if paths.is_empty() {
                    continue;
                }

                let violations = run_stalint_check(&paths);

                if !violations.is_empty() {
                    let notification = JsonRpcNotification {
                        jsonrpc: "2.0".to_string(),
                        method: "notifications/message".to_string(),
                        params: Some(serde_json::json!({
                            "level": "warning",
                            "logger": "stalint_watch",
                            "data": {
                                "message": format!(
                                    "Found {} structural violation(s)",
                                    violations.len()
                                ),
                                "violations": violations
                            }
                        })),
                    };

                    let _ = notif_tx.send(notification);
                }
            }
        }
    });
}
