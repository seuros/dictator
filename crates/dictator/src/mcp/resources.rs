//! MCP resource handlers for dictator.

use serde_json::Value;
use std::sync::{Arc, Mutex};

use super::protocol::{JsonRpcError, JsonRpcResponse};
use super::state::{ServerState, CONFIG_FILE};

/// URI for the config resource
pub const CONFIG_URI: &str = "dictator://config";
/// URI for the census resource
pub const CENSUS_URI: &str = "dictator://census";

/// Native decrees that are always available
const NATIVE_DECREES: &[(&str, &[&str], &[&str])] = &[
    ("supreme", &["*"], &[]),
    (
        "ruby",
        &["rb", "rake"],
        &["Gemfile", "Rakefile", ".rubocop.yml"],
    ),
    (
        "typescript",
        &["ts", "tsx", "js", "jsx", "mjs", "cjs"],
        &["package.json", "tsconfig.json"],
    ),
    ("python", &["py"], &["pyproject.toml", "setup.py"]),
    ("golang", &["go"], &["go.mod", "go.work"]),
    ("rust", &["rs"], &["Cargo.toml", "build.rs"]),
    ("frontmatter", &["md", "mdx"], &[]),
];

/// Handle resources/list request
pub fn handle_list_resources(id: Value) -> JsonRpcResponse {
    let resources = serde_json::json!({
        "resources": [
            {
                "uri": CONFIG_URI,
                "name": "Config",
                "description": "Current .dictate.toml configuration (parsed)",
                "mimeType": "application/json"
            },
            {
                "uri": CENSUS_URI,
                "name": "Census",
                "description": "List of all available decrees and their status",
                "mimeType": "application/json"
            }
        ]
    });

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(resources),
        error: None,
    }
}

/// Handle resources/read request
pub fn handle_read_resource(
    id: Value,
    params: Option<Value>,
    watcher_state: Arc<Mutex<ServerState>>,
) -> JsonRpcResponse {
    let Some(params) = params else {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32602,
                message: "Missing params".to_string(),
                data: None,
            }),
        };
    };

    let Some(uri) = params.get("uri").and_then(|v| v.as_str()) else {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32602,
                message: "Missing uri parameter".to_string(),
                data: None,
            }),
        };
    };

    match uri {
        CONFIG_URI => read_config_resource(id),
        CENSUS_URI => read_census_resource(id, watcher_state),
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32002,
                message: format!("Resource not found: {uri}"),
                data: None,
            }),
        },
    }
}

/// Read the config resource - returns raw TOML
fn read_config_resource(id: Value) -> JsonRpcResponse {
    let cwd = std::env::current_dir().unwrap_or_default();
    let config_path = cwd.join(CONFIG_FILE);

    let (mime_type, text) = if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(raw) => ("application/toml", raw),
            Err(e) => ("text/plain", format!("# Error reading config: {e}")),
        }
    } else {
        ("text/plain", "# No .dictate.toml found".to_string())
    };

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::json!({
            "contents": [{
                "uri": CONFIG_URI,
                "mimeType": mime_type,
                "text": text
            }]
        })),
        error: None,
    }
}

/// Read the census resource
fn read_census_resource(id: Value, watcher_state: Arc<Mutex<ServerState>>) -> JsonRpcResponse {
    // Ensure config is loaded
    watcher_state.lock().unwrap().ensure_config_loaded();

    let dictate_config = dictator_core::DictateConfig::load_default();

    let mut native_decrees = Vec::new();
    for (name, extensions, filenames) in NATIVE_DECREES {
        let (enabled, has_linter) = dictate_config.as_ref().map_or_else(
            || (*name == "supreme", false),
            |cfg| {
                let settings = cfg.decree.get(*name);
                let enabled =
                    *name == "supreme" || settings.is_some_and(|s| s.enabled != Some(false));
                let has_linter = settings.is_some_and(|s| s.linter.is_some());
                (enabled, has_linter)
            },
        );

        native_decrees.push(serde_json::json!({
            "name": name,
            "type": "native",
            "enabled": enabled,
            "extensions": extensions,
            "filenames": filenames,
            "hasExternalLinter": has_linter
        }));
    }

    // WASM decrees
    let mut wasm_decrees = Vec::new();
    if let Some(ref cfg) = dictate_config {
        for (name, settings) in &cfg.decree {
            if settings.path.is_some()
                && !NATIVE_DECREES.iter().any(|(n, _, _)| *n == name.as_str())
            {
                let path = settings.path.as_ref().unwrap();
                let exists = std::path::Path::new(path).exists();
                wasm_decrees.push(serde_json::json!({
                    "name": name,
                    "type": "wasm",
                    "enabled": exists && settings.enabled != Some(false),
                    "path": path,
                    "exists": exists
                }));
            }
        }
    }

    // External linters
    let mut linters = Vec::new();
    if let Some(ref cfg) = dictate_config {
        for (decree_name, settings) in &cfg.decree {
            if let Some(ref linter) = settings.linter {
                let available = super::utils::command_available(&linter.command);
                linters.push(serde_json::json!({
                    "decree": decree_name,
                    "command": linter.command,
                    "available": available
                }));
            }
        }
    }

    let content = serde_json::json!({
        "nativeDecrees": native_decrees,
        "wasmDecrees": wasm_decrees,
        "externalLinters": linters
    });

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::json!({
            "contents": [{
                "uri": CENSUS_URI,
                "mimeType": "application/json",
                "text": serde_json::to_string(&content).unwrap_or_default()
            }]
        })),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_list_resources() {
        let response = handle_list_resources(serde_json::json!(1));

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let resources = result["resources"].as_array().unwrap();

        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0]["uri"], CONFIG_URI);
        assert_eq!(resources[1]["uri"], CENSUS_URI);
    }

    #[test]
    fn test_handle_read_resource_missing_params() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let response = handle_read_resource(serde_json::json!(1), None, state);

        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32602);
    }

    #[test]
    fn test_handle_read_resource_missing_uri() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let params = Some(serde_json::json!({}));
        let response = handle_read_resource(serde_json::json!(1), params, state);

        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32602);
    }

    #[test]
    fn test_handle_read_resource_unknown_uri() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let params = Some(serde_json::json!({"uri": "dictator://unknown"}));
        let response = handle_read_resource(serde_json::json!(1), params, state);

        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32002);
    }

    #[test]
    fn test_handle_read_config_resource() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let params = Some(serde_json::json!({"uri": CONFIG_URI}));
        let response = handle_read_resource(serde_json::json!(1), params, state);

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let contents = result["contents"].as_array().unwrap();
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0]["uri"], CONFIG_URI);
    }

    #[test]
    fn test_handle_read_census_resource() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        let params = Some(serde_json::json!({"uri": CENSUS_URI}));
        let response = handle_read_resource(serde_json::json!(1), params, state);

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let contents = result["contents"].as_array().unwrap();
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0]["uri"], CENSUS_URI);
    }
}
