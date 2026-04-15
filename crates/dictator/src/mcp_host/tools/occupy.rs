//! OccupyTool - Initialize .dictate.toml configuration

use mcp_host::prelude::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::mcp::handlers::handle_occupy;
use crate::mcp::state::ServerState;
use crate::mcp_host::config_exists;

use super::{extract_tool_result, pretty_result_output, spawn_notification_forwarder};

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

            let string_tx = spawn_notification_forwarder(self.notification_tx.clone());
            let response = handle_occupy(Value::Null, Arc::clone(&self.state), string_tx);
            let result = extract_tool_result(response, "occupy")?;
            Ok(pretty_result_output(&result))
        })
    }
}
