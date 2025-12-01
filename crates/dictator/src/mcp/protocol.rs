//! JSON-RPC 2.0 protocol types and version handling.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields used by serde deserialization
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error object
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Implementation info (client/server)
#[derive(Debug, Deserialize, Serialize)]
pub struct Implementation {
    pub name: String,
    pub version: String,
}

/// Minimum supported client versions
pub const MIN_CLIENT_VERSIONS: &[(&str, &str)] =
    &[("claude-code", "2.0.56"), ("codex-mcp-client", "0.63.0")];

/// Client info from MCP initialize
#[derive(Debug, Clone, Default)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

impl ClientInfo {
    /// Check if client version meets minimum requirements
    pub fn is_supported(&self) -> bool {
        for (name, min_version) in MIN_CLIENT_VERSIONS {
            if self.name == *name {
                return version_gte(&self.version, min_version);
            }
        }
        // Unknown clients are allowed (for now)
        true
    }
}

/// Compare semantic versions (simple: major.minor.patch)
pub fn version_gte(version: &str, min: &str) -> bool {
    let parse = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = v.split('.').filter_map(|p| p.parse().ok()).collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };
    let (v_maj, v_min, v_patch) = parse(version);
    let (m_maj, m_min, m_patch) = parse(min);
    (v_maj, v_min, v_patch) >= (m_maj, m_min, m_patch)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== version_gte tests ==========

    #[test]
    fn test_version_gte_equal() {
        assert!(version_gte("1.0.0", "1.0.0"));
        assert!(version_gte("2.0.56", "2.0.56"));
    }

    #[test]
    fn test_version_gte_major_greater() {
        assert!(version_gte("2.0.0", "1.0.0"));
        assert!(version_gte("3.0.0", "2.9.99"));
    }

    #[test]
    fn test_version_gte_minor_greater() {
        assert!(version_gte("1.2.0", "1.1.0"));
        assert!(version_gte("1.10.0", "1.9.0"));
    }

    #[test]
    fn test_version_gte_patch_greater() {
        assert!(version_gte("1.0.2", "1.0.1"));
        assert!(version_gte("2.0.56", "2.0.55"));
    }

    #[test]
    fn test_version_gte_less_than() {
        assert!(!version_gte("1.0.0", "2.0.0"));
        assert!(!version_gte("2.0.55", "2.0.56"));
        assert!(!version_gte("1.9.0", "2.0.0"));
    }

    #[test]
    fn test_version_gte_partial_versions() {
        // Missing patch version should default to 0
        assert!(version_gte("1.0", "1.0.0"));
        assert!(version_gte("1.0.0", "1.0"));
        assert!(version_gte("2.0", "1.9.9"));
    }

    #[test]
    fn test_version_gte_single_component() {
        assert!(version_gte("2", "1.0.0"));
        assert!(version_gte("1.0.0", "1"));
    }

    // ========== ClientInfo tests ==========

    #[test]
    fn test_client_info_supported_claude_code() {
        let client = ClientInfo {
            name: "claude-code".to_string(),
            version: "2.0.56".to_string(),
        };
        assert!(client.is_supported());

        let client_newer = ClientInfo {
            name: "claude-code".to_string(),
            version: "2.1.0".to_string(),
        };
        assert!(client_newer.is_supported());
    }

    #[test]
    fn test_client_info_unsupported_old_claude_code() {
        let client = ClientInfo {
            name: "claude-code".to_string(),
            version: "2.0.55".to_string(),
        };
        assert!(!client.is_supported());

        let client_very_old = ClientInfo {
            name: "claude-code".to_string(),
            version: "1.0.0".to_string(),
        };
        assert!(!client_very_old.is_supported());
    }

    #[test]
    fn test_client_info_supported_codex() {
        let client = ClientInfo {
            name: "codex-mcp-client".to_string(),
            version: "0.63.0".to_string(),
        };
        assert!(client.is_supported());

        let client_newer = ClientInfo {
            name: "codex-mcp-client".to_string(),
            version: "1.0.0".to_string(),
        };
        assert!(client_newer.is_supported());
    }

    #[test]
    fn test_client_info_unsupported_old_codex() {
        let client = ClientInfo {
            name: "codex-mcp-client".to_string(),
            version: "0.62.9".to_string(),
        };
        assert!(!client.is_supported());
    }

    #[test]
    fn test_client_info_unknown_client_allowed() {
        let client = ClientInfo {
            name: "unknown-client".to_string(),
            version: "0.0.1".to_string(),
        };
        assert!(client.is_supported());
    }

    // ========== JSON-RPC response structure tests ==========

    #[test]
    fn test_jsonrpc_response_success() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            result: Some(serde_json::json!({"key": "value"})),
            error: None,
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert!(json.get("result").is_some());
        assert!(json.get("error").is_none()); // skip_serializing_if works
    }

    #[test]
    fn test_jsonrpc_response_error() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: "Invalid request".to_string(),
                data: None,
            }),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["error"]["code"], -32600);
        assert_eq!(json["error"]["message"], "Invalid request");
        assert!(json.get("result").is_none()); // skip_serializing_if works
    }
}
