//! OccupyTool - Initialize .dictate.toml configuration

use mcp_host::prelude::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::mcp::handlers::handle_occupy;
use crate::mcp::state::ServerState;
use crate::mcp_host::config_exists;

use super::{extract_tool_result, result_output};

/// Initialize .dictate.toml and announce catalog changes through gated APIs.
pub struct OccupyTool {
    pub state: Arc<Mutex<ServerState>>,
    pub broadcaster: Broadcaster,
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
            "properties": {
                "workspace": {
                    "type": "string",
                    "description": "Workspace root to write .dictate.toml into (default: CWD)"
                }
            },
            "required": []
        })
    }

    fn is_visible(&self, _ctx: &VisibilityContext) -> bool {
        !config_exists()
    }

    fn execute<'a>(&'a self, ctx: ExecutionContext<'a>) -> ToolFuture<'a> {
        Box::pin(async move {
            let mut ctx = ctx;
            ctx.logger.info("Initializing .dictate.toml...");

            let can_write = self.state.lock().unwrap().can_write;
            if !can_write {
                return Err(ToolError::Execution(
                    "Write operations disabled in read-only mode".to_string(),
                ));
            }

            let args = Some(ctx.params.take());
            let response = handle_occupy(
                Value::Null,
                args,
                Arc::clone(&self.state),
                ctx.client_name(),
            );
            let result = extract_tool_result(response, "occupy")?;
            self.broadcaster.broadcast(ListChanged::Tools);
            self.broadcaster.broadcast(ListChanged::Resources);
            Ok(result_output(&result))
        })
    }
}
