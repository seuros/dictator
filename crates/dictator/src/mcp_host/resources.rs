//! Resource implementations - macro-based resources here, manual resources in submodules

use mcp_host::prelude::*;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::mcp::resources::{CENSUS_URI, CONFIG_URI, handle_read_resource};
use crate::mcp::state::ServerState;
use crate::mcp_host::config_exists;

/// URI for the mood resource
pub const MOOD_URI: &str = "dictator://mood";

/// Dictator resources using macro-based registration
pub struct DictatorResources {
    pub state: Arc<Mutex<ServerState>>,
}

fn read_text_resource(
    uri: &'static str,
    handler_name: &str,
    state: &Arc<Mutex<ServerState>>,
) -> Result<String, ResourceError> {
    let params = serde_json::json!({ "uri": uri });
    let response = handle_read_resource(Value::Null, Some(params), Arc::clone(state));

    if let Some(error) = response.error {
        return Err(ResourceError::Read(error.message));
    }

    response
        .result
        .ok_or_else(|| {
            ResourceError::Read(format!("No result from {handler_name} resource handler"))
        })?
        .get("contents")
        .and_then(|contents| contents.get(0))
        .and_then(|item| item.get("text"))
        .and_then(|text| text.as_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            ResourceError::Read(format!("Missing text in {handler_name} resource handler"))
        })
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
        let text = read_text_resource(CONFIG_URI, "config", &self.state)?;
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
        let text = read_text_resource(CENSUS_URI, "census", &self.state)?;
        Ok(vec![text_resource(CENSUS_URI, text)])
    }

    /// The Dictator's current disposition toward the codebase. Subscribe to be
    /// notified when the mood shifts.
    #[mcp_resource(
        name = "Mood",
        uri = "dictator://mood",
        mime_type = "application/json",
        subscribable = true
    )]
    async fn mood(&self, _ctx: Ctx<'_>) -> ResourceResult {
        let (mood, violations) = {
            let state = self
                .state
                .lock()
                .map_err(|_| ResourceError::Read("state lock poisoned".to_string()))?;
            (state.mood(), state.last_violation_total)
        };
        let text = serde_json::json!({
            "mood": mood.label(),
            "violations": violations,
            "proclamation": mood.proclamation(),
        })
        .to_string();
        Ok(vec![text_resource(MOOD_URI, text)])
    }
}
