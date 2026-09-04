//! StalintWatchTool - Start watching paths for structural violations

use mcp_host::prelude::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::mcp::handlers::handle_stalint_watch;
use crate::mcp::state::ServerState;
use crate::mcp_host::config_exists;

use super::{
    extract_tool_result, pretty_result_output, spawn_notification_forwarder,
    stalint_unwatch::StalintUnwatchTool,
};

/// Watch files for structural changes
pub struct StalintWatchTool {
    pub state: Arc<Mutex<ServerState>>,
    pub notification_tx: NotificationSender,
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

            let string_tx = spawn_notification_forwarder(self.notification_tx.clone());
            let response = handle_stalint_watch(
                Value::Null,
                Some(ctx.params),
                Arc::clone(&self.state),
                string_tx,
            );
            let result = extract_tool_result(response, "stalint_watch")?;

            // Swap tools: hide watch, show unwatch
            ctx.session.batch(|batch| {
                batch.hide_tool("stalint_watch");
                batch.add_tool(Arc::new(StalintUnwatchTool {
                    state: Arc::clone(&self.state),
                    notification_tx: self.notification_tx.clone(),
                }));
            });

            ctx.logger.info("Watch started. stalint_unwatch now available.");

            Ok(pretty_result_output(&result))
        })
    }
}
