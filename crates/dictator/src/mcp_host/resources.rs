//! Resource implementations using mcp-host macros

use mcp_host::prelude::*;
use std::sync::{Arc, Mutex};

use super::config_exists;
use crate::mcp::resources::{CENSUS_URI, CONFIG_URI, handle_read_resource};
use crate::mcp::state::ServerState;

/// Dictator resources using macro-based registration
pub struct DictatorResources {
    pub state: Arc<Mutex<ServerState>>,
}

#[mcp_router]
impl DictatorResources {
    /// Current .dictate.toml configuration (parsed)
    #[mcp_resource(
        name = "Config",
        uri = "dictator://config",
        mime_type = "application/json",
        visible = "config_exists()"
    )]
    async fn config(&self, _ctx: Ctx<'_>) -> ResourceResult {
        let params = serde_json::json!({"uri": CONFIG_URI});
        let response = handle_read_resource(
            serde_json::Value::Null,
            Some(params),
            Arc::clone(&self.state),
        );

        if let Some(error) = response.error {
            return Err(ResourceError::Read(error.message));
        }

        let result = response.result.ok_or_else(|| {
            ResourceError::Read("No result from config resource handler".to_string())
        })?;

        let text = result
            .get("contents")
            .and_then(|c| c.get(0))
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        Ok(vec![text_resource(CONFIG_URI, text)])
    }

    /// List of all available decrees and their status
    #[mcp_resource(
        name = "Census",
        uri = "dictator://census",
        mime_type = "application/json",
        visible = "config_exists()"
    )]
    async fn census(&self, _ctx: Ctx<'_>) -> ResourceResult {
        let params = serde_json::json!({"uri": CENSUS_URI});
        let response = handle_read_resource(
            serde_json::Value::Null,
            Some(params),
            Arc::clone(&self.state),
        );

        if let Some(error) = response.error {
            return Err(ResourceError::Read(error.message));
        }

        let result = response.result.ok_or_else(|| {
            ResourceError::Read("No result from census resource handler".to_string())
        })?;

        let text = result
            .get("contents")
            .and_then(|c| c.get(0))
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        Ok(vec![text_resource(CENSUS_URI, text)])
    }
}
