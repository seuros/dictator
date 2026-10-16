//! StalintWatchTool - Start watching paths for structural violations

use mcp_host::prelude::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::mcp::handlers::handle_stalint_watch;
use crate::mcp::state::ServerState;
use crate::mcp_host::config_exists;

use super::{extract_tool_result, result_output, stalint_unwatch::StalintUnwatchTool};

/// Watch files for structural changes
pub struct StalintWatchTool {
    pub state: Arc<Mutex<ServerState>>,
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
            "properties": {
                "paths": {"type": "array", "items": {"type": "string"}}
            },
            "required": ["paths"]
        })
    }

    fn is_visible(&self, _ctx: &VisibilityContext) -> bool {
        config_exists()
    }

    fn execute<'a>(&'a self, ctx: ExecutionContext<'a>) -> ToolFuture<'a> {
        Box::pin(async move {
            let mut ctx = ctx;
            ctx.logger.info("Starting path watch...");

            let response = handle_stalint_watch(
                Value::Null,
                Some(ctx.params.take()),
                Arc::clone(&self.state),
            );
            let result = extract_tool_result(response, "stalint_watch")?;

            // Swap tools: hide watch, show unwatch
            ctx.session
                .batch(|batch| {
                    batch.add_tool(Arc::new(StalintUnwatchTool {
                        state: Arc::clone(&self.state),
                    }))?;
                    batch.hide_tool("stalint_watch");
                    Ok::<_, ToolRegistrationError>(())
                })
                .map_err(|error| ToolError::Internal(error.to_string()))?;

            ctx.logger
                .info("Watch started. stalint_unwatch now available.");

            Ok(result_output(&result))
        })
    }
}
