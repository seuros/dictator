//! StalintUnwatchTool - Stop watching paths

use mcp_host::prelude::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::mcp::handlers::handle_stalint_unwatch;
use crate::mcp::state::ServerState;
use crate::mcp_host::config_exists;

use super::{extract_tool_result, pretty_result_output, spawn_notification_forwarder};

/// Unwatch files
pub struct StalintUnwatchTool {
    pub state: Arc<Mutex<ServerState>>,
    pub notification_tx: NotificationSender,
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

            let string_tx = spawn_notification_forwarder(self.notification_tx.clone());
            let response = handle_stalint_unwatch(Value::Null, Arc::clone(&self.state), string_tx);
            let result = extract_tool_result(response, "stalint_unwatch")?;

            // Swap tools: hide unwatch, show watch
            ctx.session.batch(|batch| {
                batch.remove_tool("stalint_unwatch");
                batch.unhide_tool("stalint_watch");
            });

            ctx.logger.info("Watch stopped. stalint_watch now available.");

            Ok(pretty_result_output(&result))
        })
    }
}
