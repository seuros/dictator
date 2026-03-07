//! StalintWatchTool - Start watching paths for structural violations

use mcp_host::prelude::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::mcp::handlers::handle_stalint_watch;
use crate::mcp::state::ServerState;
use crate::mcp_host::config_exists;

use super::stalint_unwatch::StalintUnwatchTool;

/// Watch files for structural changes
pub struct StalintWatchTool {
    pub state: Arc<Mutex<ServerState>>,
    pub notification_tx: mpsc::UnboundedSender<JsonRpcNotification>,
}

impl Tool for StalintWatchTool {
    fn name(&self) -> &str {
        "stalint_watch"
    }

    fn description(&self) -> Option<&str> {
        Some("Start watching paths for structural violations")
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn is_visible(&self, _ctx: &VisibilityContext) -> bool {
        config_exists()
    }

    fn execute<'a>(&'a self, ctx: ExecutionContext<'a>) -> ToolFuture<'a> {
        Box::pin(async move {
            ctx.logger.info("Starting path watch...");

            let (string_tx, mut string_rx) = tokio::sync::mpsc::channel::<String>(100);

            let notification_tx = self.notification_tx.clone();
            tokio::spawn(async move {
                while let Some(notif_str) = string_rx.recv().await {
                    if let Ok(notif) = serde_json::from_str::<JsonRpcNotification>(&notif_str)
                        && notification_tx.send(notif).is_err()
                    {
                        break;
                    }
                }
            });

            let response = handle_stalint_watch(
                Value::Null,
                Some(ctx.params),
                Arc::clone(&self.state),
                string_tx,
            );

            if let Some(error) = response.error {
                return Err(ToolError::Execution(error.message));
            }

            let result = response.result.ok_or_else(|| {
                ToolError::Execution("No result from stalint_watch handler".to_string())
            })?;

            // Swap tools: hide watch, show unwatch
            ctx.session.batch(|batch| {
                batch.hide_tool("stalint_watch");
                batch.add_tool(Arc::new(StalintUnwatchTool {
                    state: Arc::clone(&self.state),
                    notification_tx: self.notification_tx.clone(),
                }));
            });

            ctx.logger
                .info("Watch started. stalint_unwatch now available.");

            Ok(ToolOutput::text(serde_json::to_string_pretty(&result).unwrap_or_default()))
        })
    }
}
