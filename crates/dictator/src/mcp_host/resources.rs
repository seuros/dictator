//! Resource implementations using mcp-host framework

use async_trait::async_trait;
use mcp_host::prelude::*;
use std::sync::{Arc, Mutex};

use crate::mcp::resources::{handle_read_resource, CONFIG_URI, CENSUS_URI};
use crate::mcp::state::ServerState;

/// Config resource - .dictate.toml configuration
pub struct ConfigResource {
    pub state: Arc<Mutex<ServerState>>,
}

#[async_trait]
impl Resource for ConfigResource {
    fn uri(&self) -> &str {
        CONFIG_URI
    }

    fn name(&self) -> &'static str {
        "Config"
    }

    fn description(&self) -> Option<&str> {
        Some("Current .dictate.toml configuration (parsed)")
    }

    fn mime_type(&self) -> Option<&str> {
        Some("application/json")
    }

    async fn read(&self, _ctx: ExecutionContext<'_>) -> Result<Vec<ResourceContent>, ResourceError> {
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

        // Extract text from contents array with safe navigation
        let text = result
            .get("contents")
            .and_then(|c| c.get(0))
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        Ok(vec![self.text_content(&text)])
    }
}

/// Census resource - list of available decrees
pub struct CensusResource {
    pub state: Arc<Mutex<ServerState>>,
}

#[async_trait]
impl Resource for CensusResource {
    fn uri(&self) -> &str {
        CENSUS_URI
    }

    fn name(&self) -> &'static str {
        "Census"
    }

    fn description(&self) -> Option<&str> {
        Some("List of all available decrees and their status")
    }

    fn mime_type(&self) -> Option<&str> {
        Some("application/json")
    }

    async fn read(&self, _ctx: ExecutionContext<'_>) -> Result<Vec<ResourceContent>, ResourceError> {
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

        // Extract text from contents array with safe navigation
        let text = result
            .get("contents")
            .and_then(|c| c.get(0))
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

        Ok(vec![self.text_content(&text)])
    }
}
