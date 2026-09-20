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
use crate::mcp::utils::{GitScope, git_changed_files, to_json_string_pretty};
use crate::mcp_host::config_exists;

/// Arguments for the stalint tool
#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
pub struct StalintParams {
    /// Lint only files staged for commit (default: all uncommitted changes)
    #[serde(default)]
    pub staged: bool,
    /// Workspace root to scope linting to (default: client roots or CWD)
    #[serde(default)]
    pub workspace: Option<String>,
}

/// Arguments for the dictator (auto-fix) tool
#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
pub struct DictatorParams {
    /// Workspace root to scope auto-fix to (default: client roots or CWD)
    #[serde(default)]
    pub workspace: Option<String>,
}

/// Canonicalize and validate a caller-supplied workspace override.
fn validate_workspace(workspace: Option<&str>) -> Result<Option<String>, ToolError> {
    let Some(ws) = workspace else {
        return Ok(None);
    };
    let canonical = std::fs::canonicalize(ws)
        .map_err(|e| ToolError::Execution(format!("workspace '{ws}' is not accessible: {e}")))?;
    if !canonical.is_dir() {
        return Err(ToolError::Execution(format!(
            "workspace '{ws}' is not a directory"
        )));
    }
    Ok(Some(canonical.to_string_lossy().into_owned()))
}

/// Resolve the lint/fix scope from client roots or CWD, narrowed to uncommitted files
/// wherever the resolved directory sits inside a git repo.
///
/// - Client advertises roots capability → request roots list
///   - Non-empty → use root URIs converted to filesystem paths
///   - Empty → return `None` (caller should hide tools and bail out)
/// - Client has no roots capability → use CWD
/// - Each resolved directory that's inside a git repo is replaced by its uncommitted
///   (staged/unstaged/untracked) file list; directories outside a repo are kept as-is,
///   preserving the old whole-tree behavior there.
/// - Returns `Some(vec![])` when every resolved directory is a clean git repo — callers
///   must treat that as "nothing to do", not as "no scope, so scan everything".
async fn resolve_paths(
    ctx: &Ctx<'_>,
    scope: GitScope,
    workspace: Option<&str>,
) -> Option<Vec<String>> {
    const ROOTS_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

    fn cwd_fallback() -> Option<Vec<String>> {
        std::env::current_dir()
            .ok()
            .map(|p| vec![p.to_string_lossy().to_string()])
    }

    let dirs = if let Some(ws) = workspace {
        vec![ws.to_string()]
    } else if ctx.supports_roots() {
        let requester = ctx.client_requester();
        let roots = match requester {
            None => None,
            Some(requester) => {
                match tokio::time::timeout(ROOTS_TIMEOUT, requester.request_roots(None)).await {
                    Ok(Ok(roots)) => Some(roots),
                    // Request errored or timed out: client claims roots support but
                    // didn't deliver, so don't hang or lint the whole workspace —
                    // fall back to cwd instead of bailing to "scan everything".
                    Ok(Err(_)) | Err(_) => None,
                }
            }
        };

        match roots {
            // Client answered with an explicit empty list: no scope, caller must bail.
            Some(roots) if roots.is_empty() => return None,
            Some(roots) => roots
                .iter()
                .map(|r| {
                    // Strip file:// or file:/// prefix; leave other URIs as-is
                    if let Some(p) = r.uri.strip_prefix("file://") {
                        p.to_string()
                    } else {
                        r.uri.clone()
                    }
                })
                .collect(),
            None => cwd_fallback()?,
        }
    } else {
        cwd_fallback()?
    };

    let mut scoped = Vec::new();
    let mut any_git = false;
    for dir in &dirs {
        match git_changed_files(std::path::Path::new(dir), scope) {
            Some(files) => {
                any_git = true;
                scoped.extend(files.into_iter().map(|p| p.to_string_lossy().into_owned()));
            }
            None => scoped.push(dir.clone()),
        }
    }

    Some(if any_git { scoped } else { dirs })
}

pub(super) fn spawn_notification_forwarder(
    notification_tx: NotificationSender,
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
    async fn stalint(&self, ctx: Ctx<'_>, params: Parameters<StalintParams>) -> ToolResult {
        let workspace = validate_workspace(params.0.workspace.as_deref())?;
        let scope = if params.0.staged {
            GitScope::Staged
        } else {
            GitScope::Uncommitted
        };
        let paths = match resolve_paths(&ctx, scope, workspace.as_deref()).await {
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

        if paths.is_empty() {
            let scope_word = if params.0.staged {
                "staged"
            } else {
                "uncommitted"
            };
            return Ok(ToolOutput::text(format!(
                "Working tree is clean — no {scope_word} files to lint."
            )));
        }

        if params.0.staged {
            let espionage = paths.iter().any(|p| {
                dictator_core::classified::is_classified(camino::Utf8Path::new(p.as_str()))
            });
            self.state
                .lock()
                .unwrap()
                .record_classified_staged(espionage);
        }

        let args = Some(serde_json::json!({ "paths": paths, "workspace": workspace }));
        let response = handle_stalint(Value::Null, args, Arc::clone(&self.state));
        let mut result = extract_tool_result(response, "stalint")?;

        // Staged classified files are a commit-in-progress leak: escalate
        if params.0.staged
            && let Some(violations) = result
                .pointer_mut("/structuredContent/violations")
                .and_then(Value::as_array_mut)
        {
            for violation in violations {
                if violation["rule"] == dictator_core::classified::CLASSIFIED_RULE {
                    violation["message"] = Value::String(
                        "classified material staged for commit — unstage immediately".to_string(),
                    );
                }
            }
        }
        Ok(pretty_result_output(&result))
    }

    /// Auto-fix structural violations (requires write permissions)
    #[mcp_tool(
        name = "dictator",
        title = "Dictator Auto-Fix",
        visible = "config_exists()",
        destructive = true
    )]
    async fn dictator(&self, ctx: Ctx<'_>, params: Parameters<DictatorParams>) -> ToolResult {
        let can_write = self.state.lock().unwrap().can_write;
        if !can_write {
            return Err(ToolError::Execution(
                "Write operations disabled in read-only mode".to_string(),
            ));
        }

        let workspace = validate_workspace(params.0.workspace.as_deref())?;
        let paths = match resolve_paths(&ctx, GitScope::Uncommitted, workspace.as_deref()).await {
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

        if paths.is_empty() {
            return Ok(ToolOutput::text("Working tree is clean — nothing to fix."));
        }

        if let Some(requester) = ctx.client_requester()
            && requester.supports_elicitation()
        {
            let lint_summary = {
                let args = Some(serde_json::json!({ "paths": paths, "workspace": workspace }));
                let response = handle_stalint(Value::Null, args, Arc::clone(&self.state));
                response
                    .result
                    .and_then(|r| serde_json::to_string_pretty(&r).ok())
                    .unwrap_or_else(|| "unknown violations".to_string())
            };
            let message = format!(
                "dictator will auto-fix the following \
                 violations:\n\n{lint_summary}\n\nConfirm?"
            );

            let schema = ElicitationSchema::builder()
                .optional_bool("confirm", false)
                .build_unchecked();

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
                return Err(ToolError::Execution(
                    "Operation cancelled by user".to_string(),
                ));
            }
        }

        let args = Some(serde_json::json!({ "paths": paths, "workspace": workspace }));
        let response = handle_dictator(Value::Null, args, Arc::clone(&self.state));
        let result = extract_tool_result(response, "dictator")?;
        Ok(pretty_result_output(&result))
    }
}
