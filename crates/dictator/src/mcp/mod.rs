//! MCP (Model Context Protocol) server implementation for Dictator.
//!
//! Implements MCP 2025-06-18 spec over JSON-RPC 2.0 stdio transport.

mod fixers;
mod handlers;
mod linters;
mod logging;
mod progress;
mod protocol;
mod regime;
mod resources;
mod state;
mod tools;
mod utils;

use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;

use protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use regime::run_stalint_check;
use resources::{handle_list_resources, handle_read_resource};
use state::{STALINT_CHECK_TIMEOUT_SECS, ServerState, WATCHER_CHECK_INTERVAL_SECS};
use tools::{handle_call_tool, handle_initialize, handle_list_tools};
use utils::log_to_file;

/// Run the MCP server (JSON-RPC over stdio)
pub fn run() -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run_async())
}

async fn run_async() -> Result<()> {
    log_to_file("=== Dictator MCP server starting ===");
    log_to_file(&format!("Version: {}", env!("CARGO_PKG_VERSION")));
    log_to_file(&format!("PID: {}", std::process::id()));

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();

    // Channel for notifications from watcher
    let (notif_tx, mut notif_rx) = mpsc::channel::<String>(100);

    // Shared watcher state (initialize with notification channel)
    let watcher_state = Arc::new(Mutex::new(ServerState::new(notif_tx.clone())));

    // Start watcher check task
    let watcher_state_clone = Arc::clone(&watcher_state);
    let notif_tx_clone = notif_tx.clone();
    tokio::spawn(async move {
        watcher_check_loop(watcher_state_clone, notif_tx_clone).await;
    });

    let mut line = String::new();

    loop {
        tokio::select! {
            // Handle notifications from watcher
            Some(notif) = notif_rx.recv() => {
                tracing::debug!("Sending notification: {}", notif);
                if let Err(e) = stdout.write_all(notif.as_bytes()).await {
                    tracing::error!("Failed to write notification: {}", e);
                }
                if let Err(e) = stdout.write_all(b"\n").await {
                    tracing::error!("Failed to write newline: {}", e);
                }
                let _ = stdout.flush().await;
            }

            // Handle stdin input
            result = reader.read_line(&mut line) => {
                let n = result?;

                if n == 0 {
                    tracing::info!("Received EOF, shutting down");
                    break;
                }

                let trimmed = line.trim();
                if trimmed.is_empty() {
                    line.clear();
                    continue;
                }

                tracing::debug!("Received: {}", trimmed);

                let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
                    Ok(req) => req,
                    Err(e) => {
                        log_to_file(&format!("PARSE ERROR: {e}"));
                        line.clear();
                        continue;
                    }
                };

                log_to_file(&format!(">>> {}", request.method));

                // Handle notifications (no response needed)
                if request.id.is_none() {
                    let state = Arc::clone(&watcher_state);
                    handle_notification(&request, state, notif_tx.clone()).await;
                    line.clear();
                    continue;
                }

                // Handle requests (response required)
                let state = Arc::clone(&watcher_state);
                let response = handle_request(request, state, notif_tx.clone());
                let response_json = serde_json::to_string(&response)?;

                tracing::debug!("Sending: {}", response_json);
                stdout.write_all(response_json.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;

                line.clear();
            }
        }
    }

    Ok(())
}

/// Background loop that checks watched paths
async fn watcher_check_loop(state: Arc<Mutex<ServerState>>, notif_tx: mpsc::Sender<String>) {
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

            // Run stalint on watched paths
            let violations = run_stalint_check(&paths);

            if !violations.is_empty() {
                let notification = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "notifications/message",
                    "params": {
                        "level": "warning",
                        "logger": "stalint_watch",
                        "data": {
                            "message": format!(
                                "Found {} structural violation(s)",
                                violations.len()
                            ),
                            "violations": violations
                        }
                    }
                });

                let _ = notif_tx.send(notification.to_string()).await;
            }
        }
    }
}

async fn handle_notification(
    req: &JsonRpcRequest,
    watcher_state: Arc<Mutex<ServerState>>,
    notif_tx: mpsc::Sender<String>,
) {
    log_to_file(&format!("NOTIFICATION: {}", req.method));
    if let Some(params) = &req.params {
        log_to_file(&format!("  params: {params}"));
    }

    match req.method.as_str() {
        "notifications/initialized" => {
            log_to_file("Client initialized");
        }
        "notifications/cancelled" => {
            log_to_file("Request cancelled");
        }
        "codex/sandbox-state/update" => {
            handle_sandbox_state_update(req.params.clone(), watcher_state, notif_tx).await;
        }
        _ => {
            log_to_file(&format!("Unknown notification: {}", req.method));
        }
    }
}

/// Handle Codex sandbox state update notification
async fn handle_sandbox_state_update(
    params: Option<Value>,
    watcher_state: Arc<Mutex<ServerState>>,
    notif_tx: mpsc::Sender<String>,
) {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct SandboxState {
        sandbox_policy: SandboxPolicy,
    }

    #[derive(Deserialize)]
    #[serde(tag = "type", rename_all = "kebab-case")]
    #[allow(dead_code)] // Fields used by serde for variant matching
    enum SandboxPolicy {
        DangerFullAccess,
        ReadOnly,
        #[serde(rename = "workspace-write")]
        WorkspaceWrite {
            #[serde(default)]
            writable_roots: Vec<String>,
            #[serde(default)]
            network_access: bool,
        },
    }

    let Some(params) = params else {
        tracing::warn!("sandbox-state/update: missing params");
        return;
    };

    let state: SandboxState = match serde_json::from_value(params) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("sandbox-state/update: failed to parse: {}", e);
            return;
        }
    };

    let policy_name = match &state.sandbox_policy {
        SandboxPolicy::ReadOnly => "read-only",
        SandboxPolicy::DangerFullAccess => "danger-full-access",
        SandboxPolicy::WorkspaceWrite { .. } => "workspace-write",
    };
    log_to_file(&format!("SANDBOX: policy = {policy_name}"));

    let new_can_write = !matches!(state.sandbox_policy, SandboxPolicy::ReadOnly);
    let old_can_write = {
        let mut s = watcher_state.lock().unwrap();
        let old = s.can_write;
        s.can_write = new_can_write;
        old
    };

    log_to_file(&format!(
        "SANDBOX: can_write {old_can_write} -> {new_can_write}"
    ));

    // Notify tools list changed if capability changed
    if old_can_write != new_can_write {
        log_to_file("SANDBOX: sending tools/list_changed notification");
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/tools/list_changed",
            "params": {}
        });
        let _ = notif_tx.send(notification.to_string()).await;
    }
}

fn handle_request(
    req: JsonRpcRequest,
    watcher_state: Arc<Mutex<ServerState>>,
    notif_tx: mpsc::Sender<String>,
) -> JsonRpcResponse {
    let id = req.id.clone().unwrap_or(Value::Null);

    match req.method.as_str() {
        "initialize" => handle_initialize(id, req.params, Arc::clone(&watcher_state)),
        "tools/list" => handle_list_tools(id, Arc::clone(&watcher_state)),
        "tools/call" => handle_call_tool(id, req.params, watcher_state, notif_tx),
        "resources/list" => handle_list_resources(id, Arc::clone(&watcher_state)),
        "resources/read" => handle_read_resource(id, req.params, watcher_state),
        "logging/setLevel" => {
            tools::handle_logging_set_level(id, req.params, Arc::clone(&watcher_state))
        }
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", req.method),
                data: None,
            }),
        },
    }
}
