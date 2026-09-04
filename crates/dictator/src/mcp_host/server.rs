//! Main MCP server using mcp-host framework

use anyhow::Result;
use mcp_host::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::mcp::regime::run_stalint_check;
use crate::mcp::state::{
    CONFIG_FILE, STALINT_CHECK_TIMEOUT_SECS, ServerState, WATCHER_CHECK_INTERVAL_SECS,
};
use crate::mcp::utils::{GitScope, files_fingerprint, git_changed_files, log_to_file};

use super::prompts::DictatorPrompts;
use super::resources::DictatorResources;
use super::tools::{DictatorTools, OccupyTool, StalintWatchTool};

/// Run the MCP server with mcp-host framework
pub fn run() -> Result<()> {
    tokio::runtime::Builder::new_multi_thread().enable_all().build()?.block_on(run_async())
}

async fn run_async() -> Result<()> {
    log_to_file("=== Dictator MCP server starting (mcp-host) ===");
    log_to_file(&format!("Version: {}", env!("CARGO_PKG_VERSION")));
    log_to_file(&format!("PID: {}", std::process::id()));

    // Build server with capabilities via builder
    let server = Arc::new(
        Server::builder("dictator", env!("CARGO_PKG_VERSION"))
            .with_title("The Dictator")
            .with_description(env!("CARGO_PKG_DESCRIPTION"))
            .with_website_url(env!("CARGO_PKG_HOMEPAGE"))
            .with_instructions("Run stalint before any commit. User expects disciplined agents.")
            .with_tools(true)
            .with_resources(true, true)
            .with_prompts(true)
            .build(),
    );

    // Get notification sender for background tasks
    let notification_tx = server.notification_sender();

    // Create shared state with proper notification channel
    let watcher_state = Arc::new(Mutex::new(ServerState::new(notification_tx.clone())));

    // Register macro-based tools (stalint, dictator) via unified router
    let tools = Arc::new(DictatorTools { state: Arc::clone(&watcher_state) });
    server.register_router(DictatorTools::router(), tools);

    // Register stateful tools (manual impl - need notification_tx)
    server.tool_registry().register(StalintWatchTool {
        state: Arc::clone(&watcher_state),
        notification_tx: notification_tx.clone(),
    });
    server.tool_registry().register(OccupyTool {
        state: Arc::clone(&watcher_state),
        notification_tx: notification_tx.clone(),
    });

    // Register macro-based resources via unified router
    let resources = Arc::new(DictatorResources { state: Arc::clone(&watcher_state) });
    server.register_router(DictatorResources::router(), resources);

    // Register macro-based prompts via unified router
    let prompts = Arc::new(DictatorPrompts);
    server.register_router(DictatorPrompts::router(), prompts);

    // Start background tasks
    start_config_watcher(Arc::clone(&watcher_state), notification_tx.clone());
    start_watcher_check_loop(Arc::clone(&watcher_state), notification_tx, Arc::clone(&server));

    // Run server with stdio transport
    let transport = StdioTransport::new();
    server.run(transport).await.map_err(|e| anyhow::anyhow!("{e}"))?;

    Ok(())
}

/// Background loop that watches .dictate.toml for changes
fn start_config_watcher(state: Arc<Mutex<ServerState>>, notif_tx: NotificationSender) {
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

        let watch_path = if config_path.exists() { config_path.clone() } else { cwd.clone() };

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

                // Send list_changed notifications
                let _ = notif_tx
                    .send(JsonRpcNotification::new("notifications/tools/list_changed", None));
                let _ = notif_tx
                    .send(JsonRpcNotification::new("notifications/resources/list_changed", None));
                let _ = notif_tx
                    .send(JsonRpcNotification::new("notifications/prompts/list_changed", None));

                log_to_file("Config changed: sent list_changed for tools/resources/prompts");
            }
        }
    });
}

/// Background loop that checks watched paths
fn start_watcher_check_loop(
    state: Arc<Mutex<ServerState>>,
    notif_tx: NotificationSender,
    server: Arc<Server>,
) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(WATCHER_CHECK_INTERVAL_SECS)).await;

            // Re-lint uncommitted files when their fingerprint changes
            if let Ok(cwd) = std::env::current_dir()
                && let Some(files) = git_changed_files(&cwd, GitScope::Uncommitted)
            {
                // Staged classified material triggers the paranoid mood
                let espionage = git_changed_files(&cwd, GitScope::Staged).is_some_and(|staged| {
                    staged.iter().any(|p| {
                        dictator_core::classified::is_classified(camino::Utf8Path::new(
                            &p.to_string_lossy(),
                        ))
                    })
                });
                state.lock().unwrap().record_classified_staged(espionage);

                let fingerprint = files_fingerprint(&files);
                let stale = { state.lock().unwrap().uncommitted_fingerprint != Some(fingerprint) };
                if stale {
                    let paths: Vec<String> =
                        files.iter().map(|p| p.to_string_lossy().into_owned()).collect();
                    let violations = run_stalint_check(&paths);
                    let mut s = state.lock().unwrap();
                    s.record_uncommitted_check(fingerprint, violations.len(), paths.len());
                }
            }

            let (mood_changed, uncommitted_changed) = {
                let mut state = state.lock().unwrap();
                (
                    std::mem::take(&mut state.mood_dirty),
                    std::mem::take(&mut state.uncommitted_dirty),
                )
            };
            if mood_changed {
                server.notify_resource_updated(super::resources::MOOD_URI);
                log_to_file("Mood shifted: notified dictator://mood subscribers");
            }
            if uncommitted_changed {
                server.notify_resource_updated(super::resources::UNCOMMITTED_URI);
                log_to_file("Uncommitted status changed: notified subscribers");
            }

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
                {
                    let mut state = state.lock().unwrap();
                    state.record_violations(violations.len());
                }

                if !violations.is_empty() {
                    let _ = notif_tx.send(JsonRpcNotification::new(
                        "notifications/message",
                        Some(serde_json::json!({
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
                    ));
                }
            }
        }
    });
}
