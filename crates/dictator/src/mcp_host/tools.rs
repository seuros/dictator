//! Tool implementations - macro-based tools here, stateful tools in submodules

mod occupy;
mod stalint_unwatch;
mod stalint_watch;

pub use occupy::OccupyTool;
pub use stalint_watch::StalintWatchTool;

use mcp_host::prelude::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::mcp::handlers::{handle_dictator, handle_stalint};
use crate::mcp::state::ServerState;
use crate::mcp_host::config_exists;

/// Simple tools using macro-based registration
pub struct DictatorTools {
    pub state: Arc<Mutex<ServerState>>,
}

#[mcp_router]
impl DictatorTools {
    /// Run structural linting checks on files (read-only analysis)
    #[mcp_tool(name = "stalint", visible = "config_exists()")]
    async fn stalint(&self, _ctx: Ctx<'_>, _params: Parameters<()>) -> ToolResult {
        let response = handle_stalint(Value::Null, None, Arc::clone(&self.state));

        if let Some(error) = response.error {
            return Err(ToolError::Execution(error.message));
        }

        let result = response
            .result
            .ok_or_else(|| ToolError::Execution("No result from stalint handler".to_string()))?;

        Ok(ToolOutput::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        ))
    }

    /// Auto-fix structural violations (requires write permissions)
    #[mcp_tool(name = "dictator", visible = "config_exists()")]
    async fn dictator(&self, _ctx: Ctx<'_>, _params: Parameters<()>) -> ToolResult {
        let can_write = self.state.lock().unwrap().can_write;
        if !can_write {
            return Err(ToolError::Execution(
                "Write operations disabled in read-only mode".to_string(),
            ));
        }

        let response = handle_dictator(Value::Null, None, Arc::clone(&self.state));

        if let Some(error) = response.error {
            return Err(ToolError::Execution(error.message));
        }

        let result = response
            .result
            .ok_or_else(|| ToolError::Execution("No result from dictator handler".to_string()))?;

        Ok(ToolOutput::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        ))
    }
}
