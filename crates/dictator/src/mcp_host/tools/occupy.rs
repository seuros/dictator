//! OccupyTool - Initialize .dictate.toml configuration

use mcp_host::prelude::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::mcp::handlers::handle_occupy;
use crate::mcp::state::ServerState;
use crate::mcp_host::config_exists;

/// Initialize .dictate.toml - needs notification_tx for list_changed
pub struct OccupyTool {
    pub state: Arc<Mutex<ServerState>>,
    pub notification_tx: mpsc::UnboundedSender<JsonRpcNotification>,
}

impl Tool for OccupyTool {
    fn name(&self) -> &str {
        "occupy"
    }

    fn description(&self) -> Option<&str> {
        Some("Initialize .dictate.toml configuration file (requires write permissions)")
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    fn is_visible(&self, _ctx: &VisibilityContext) -> bool {
        !config_exists()
    }

    fn execute<'a>(&'a self, ctx: ExecutionContext<'a>) -> ToolFuture<'a> {
        Box::pin(async move {
            ctx.logger.info("Initializing .dictate.toml...");

            let can_write = self.state.lock().unwrap().can_write;
            if !can_write {
                return Err(ToolError::Execution(
                    "Write operations disabled in read-only mode".to_string(),
                ));
            }

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

            let response = handle_occupy(Value::Null, Arc::clone(&self.state), string_tx);

            if let Some(error) = response.error {
                return Err(ToolError::Execution(error.message));
            }

            let result = response
                .result
                .ok_or_else(|| ToolError::Execution("No result from occupy handler".to_string()))?;

            let content =
                TextContent::new(serde_json::to_string_pretty(&result).unwrap_or_default());
            Ok(ToolOutput::Content(vec![Box::new(content)]))
        })
    }
}
