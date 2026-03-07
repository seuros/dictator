//! StalintUnwatchTool - Stop watching paths

use mcp_host::prelude::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::mcp::handlers::handle_stalint_unwatch;
use crate::mcp::state::ServerState;
use crate::mcp_host::config_exists;

/// Unwatch files
pub struct StalintUnwatchTool {
    pub state: Arc<Mutex<ServerState>>,
    pub notification_tx: mpsc::UnboundedSender<JsonRpcNotification>,
}

impl Tool for StalintUnwatchTool {
    fn name(&self) -> &str {
        "stalint_unwatch"
    }

    fn description(&self) -> Option<&str> {
        Some("Stop watching paths")
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
            ctx.logger.info("Stopping path watch...");

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

            let response = handle_stalint_unwatch(Value::Null, Arc::clone(&self.state), string_tx);

            if let Some(error) = response.error {
                return Err(ToolError::Execution(error.message));
            }

            let result = response.result.ok_or_else(|| {
                ToolError::Execution("No result from stalint_unwatch handler".to_string())
            })?;

            // Swap tools: hide unwatch, show watch
            ctx.session.batch(|batch| {
                batch.remove_tool("stalint_unwatch");
                batch.unhide_tool("stalint_watch");
            });

            ctx.logger
                .info("Watch stopped. stalint_watch now available.");

            let content =
                TextContent::new(serde_json::to_string_pretty(&result).unwrap_or_default());
            Ok(ToolOutput::Content(vec![Box::new(content)]))
        })
    }
}
