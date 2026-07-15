//! Tool implementations - macro-based tools here, stateful tools in submodules

mod occupy;
mod stalint_unwatch;
mod stalint_watch;

pub use occupy::OccupyTool;
pub use stalint_watch::StalintWatchTool;

use mcp_host::prelude::*;
use mcp_host::protocol::elicitation::ElicitationSchema;
use mcp_host::protocol::types::ElicitationAction;
use mcp_host::protocol::types::JsonRpcResponse;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::mcp::handlers::{handle_dictator, handle_stalint};
use crate::mcp::state::ServerState;
use crate::mcp::utils::to_json_string_pretty;
use crate::mcp_host::config_exists;

/// Resolve workspace paths from client roots or fall back to CWD.
///
/// - Client advertises roots capability → request roots list
///   - Non-empty → use root URIs converted to filesystem paths
///   - Empty → return `None` (caller should hide tools and bail out)
/// - Client has no roots capability → use CWD
async fn resolve_paths(ctx: &Ctx<'_>) -> Option<Vec<String>> {
    if ctx.supports_roots() {
        let requester = ctx.client_requester()?;
        let roots = requester.request_roots(None).await.ok()?;
        if roots.is_empty() {
            return None;
        }
        let paths = roots
            .iter()
            .map(|r| {
                // Strip file:// or file:/// prefix; leave other URIs as-is
                if let Some(p) = r.uri.strip_prefix("file://") {
                    p.to_string()
                } else {
                    r.uri.clone()
                }
            })
            .collect();
        Some(paths)
    } else {
        // Fall back to current working directory
        let cwd = std::env::current_dir().ok()?.to_string_lossy().to_string();
        Some(vec![cwd])
    }
}

pub(super) fn spawn_notification_forwarder(
    notification_tx: mpsc::UnboundedSender<JsonRpcNotification>,
) -> mpsc::Sender<String> {
    let (string_tx, mut string_rx) = mpsc::channel::<String>(100);

    tokio::spawn(async move {
        while let Some(notif_str) = string_rx.recv().await {
            if let Ok(notif) = serde_json::from_str::<JsonRpcNotification>(&notif_str)
                && notification_tx.send(notif).is_err()
            {
                break;
            }
        }
    });

    string_tx
}

pub(super) fn extract_tool_result(
    response: JsonRpcResponse,
    handler_name: &str,
) -> Result<Value, ToolError> {
    if let Some(error) = response.error {
        return Err(ToolError::Execution(error.message));
    }

    response
        .result
        .ok_or_else(|| ToolError::Execution(format!("No result from {handler_name} handler")))
}

pub(super) fn pretty_result_output(result: &Value) -> ToolOutput {
    ToolOutput::text(to_json_string_pretty(result))
}

/// Simple tools using macro-based registration
pub struct DictatorTools {
    pub state: Arc<Mutex<ServerState>>,
}

#[mcp_router]
impl DictatorTools {
    /// Run structural linting checks on files (read-only analysis)
    #[mcp_tool(
        name = "stalint",
        title = "Structural Lint",
        visible = "config_exists()",
        read_only = true,
        idempotent = true
    )]
    async fn stalint(&self, ctx: Ctx<'_>, _params: Parameters<()>) -> ToolResult {
        let paths = match resolve_paths(&ctx).await {
            Some(p) => p,
            None => {
                // Client supports roots but returned empty — hide both tools
                ctx.session.batch(|batch| {
                    batch.hide_tool("stalint");
                    batch.hide_tool("dictator");
                });
                return Ok(ToolOutput::text(
                    "No workspace roots configured. Tools hidden until roots are available.",
                ));
            }
        };

        let args = Some(serde_json::json!({ "paths": paths }));
        let response = handle_stalint(Value::Null, args, Arc::clone(&self.state));
        let result = extract_tool_result(response, "stalint")?;
        Ok(pretty_result_output(&result))
    }

    /// Auto-fix structural violations (requires write permissions)
    #[mcp_tool(
        name = "dictator",
        title = "Dictator Auto-Fix",
        visible = "config_exists()",
        destructive = true
    )]
    async fn dictator(&self, ctx: Ctx<'_>, _params: Parameters<()>) -> ToolResult {
        let can_write = self.state.lock().unwrap().can_write;
        if !can_write {
            return Err(ToolError::Execution(
                "Write operations disabled in read-only mode".to_string(),
            ));
        }

        if let Some(requester) = ctx.client_requester()
            && requester.supports_elicitation()
        {
            let lint_summary = {
                let response = handle_stalint(Value::Null, None, Arc::clone(&self.state));
                response
                    .result
                    .and_then(|r| serde_json::to_string_pretty(&r).ok())
                    .unwrap_or_else(|| "unknown violations".to_string())
            };
            let message = format!(
                "dictator will auto-fix the following \
                 violations:\n\n{lint_summary}\n\nConfirm?"
            );

            let schema =
                ElicitationSchema::builder().optional_bool("confirm", false).build_unchecked();

            let result = requester
                .request_elicitation(
                    message,
                    serde_json::to_value(&schema).unwrap_or_default(),
                    None,
                )
                .await
                .map_err(|e| ToolError::Execution(e.to_string()))?;

            let confirmed = matches!(result.action, ElicitationAction::Accept)
                && result
                    .content
                    .as_ref()
                    .and_then(|c| c.get("confirm"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

            if !confirmed {
                return Err(ToolError::Execution("Operation cancelled by user".to_string()));
            }
        }

        let paths = match resolve_paths(&ctx).await {
            Some(p) => p,
            None => {
                // Client supports roots but returned empty — hide both tools
                ctx.session.batch(|batch| {
                    batch.hide_tool("stalint");
                    batch.hide_tool("dictator");
                });
                return Ok(ToolOutput::text(
                    "No workspace roots configured. Tools hidden until roots are available.",
                ));
            }
        };

        let args = Some(serde_json::json!({ "paths": paths }));
        let response = handle_dictator(Value::Null, args, Arc::clone(&self.state));
        let result = extract_tool_result(response, "dictator")?;
        Ok(pretty_result_output(&result))
    }
}
