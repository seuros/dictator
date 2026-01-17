//! Resource implementations with dynamic visibility based on config state

use async_trait::async_trait;
use mcp_host::prelude::*;
use std::sync::{Arc, Mutex};

use super::config_exists;
use crate::mcp::resources::{CENSUS_URI, CONFIG_URI, handle_read_resource};
use crate::mcp::state::ServerState;

/// Config resource - .dictate.toml configuration
pub struct ConfigResource {
    state: Arc<Mutex<ServerState>>,
}

impl ConfigResource {
    pub const fn new(state: Arc<Mutex<ServerState>>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Resource for ConfigResource {
    fn name(&self) -> &'static str {
        "Config"
    }

    fn uri(&self) -> &str {
        CONFIG_URI
    }

    fn description(&self) -> Option<&str> {
        Some("Current .dictate.toml configuration (parsed)")
    }

    fn mime_type(&self) -> Option<&str> {
        Some("application/json")
    }

    fn is_visible(&self, _ctx: &VisibilityContext) -> bool {
        config_exists()
    }

    async fn read(
        &self,
        _ctx: ExecutionContext<'_>,
    ) -> Result<Vec<ResourceContent>, ResourceError> {
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

        Ok(vec![self.text_content(&text)])
    }
}

/// Census resource - list of available decrees
pub struct CensusResource {
    state: Arc<Mutex<ServerState>>,
}

impl CensusResource {
    pub const fn new(state: Arc<Mutex<ServerState>>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Resource for CensusResource {
    fn name(&self) -> &'static str {
        "Census"
    }

    fn uri(&self) -> &str {
        CENSUS_URI
    }

    fn description(&self) -> Option<&str> {
        Some("List of all available decrees and their status")
    }

    fn mime_type(&self) -> Option<&str> {
        Some("application/json")
    }

    fn is_visible(&self, _ctx: &VisibilityContext) -> bool {
        config_exists()
    }

    async fn read(
        &self,
        _ctx: ExecutionContext<'_>,
    ) -> Result<Vec<ResourceContent>, ResourceError> {
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

        Ok(vec![self.text_content(&text)])
    }
}
