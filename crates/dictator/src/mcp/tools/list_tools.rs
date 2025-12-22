//! Tools list request handler for MCP protocol.

use serde_json::Value;
use std::sync::{Arc, Mutex};

use crate::mcp::protocol::JsonRpcResponse;
use crate::mcp::state::ServerState;
use crate::mcp::utils::{command_available, is_git_repo};

/// Handle tools/list request
pub fn handle_list_tools(id: Value, watcher_state: Arc<Mutex<ServerState>>) -> JsonRpcResponse {
    tracing::info!("Handling tools/list");

    let (is_watching, can_write, config_exists) = {
        let mut state = watcher_state.lock().unwrap();
        state.ensure_config_loaded();
        (state.is_watching, state.can_write, state.config.is_some())
    };

    // No config? Only show occupy tool
    if !config_exists && can_write {
        let tools = serde_json::json!({
            "tools": [serde_json::json!({
                "name": "occupy",
                "title": "Initialize Config",
                "description": "Initialize .dictate.toml with default configuration.",
                "annotations": {
                    "title": "Initialize Config",
                    "readOnlyHint": false,
                    "destructiveHint": false,
                    "idempotentHint": true,
                    "openWorldHint": false
                },
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            })]
        });

        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(tools),
            error: None,
        };
    }

    // Watch tool changes based on state
    let watch_tool = if is_watching {
        serde_json::json!({
            "name": "stalint_unwatch",
            "title": "Stop Watcher",
            "description": "Stop watching for file changes.",
            "annotations": {
                "title": "Stop Watcher",
                "readOnlyHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        })
    } else {
        serde_json::json!({
            "name": "stalint_watch",
            "title": "File Watcher",
            "description": "Watch paths for file changes. Runs stalint every 60s \
                            when changes detected and sends notifications with violations.",
            "annotations": {
                "title": "File Watcher",
                "readOnlyHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "File or directory paths to watch"
                    }
                },
                "required": ["paths"]
            }
        })
    };

    // Build tools list dynamically based on capabilities
    let mut tool_list = vec![serde_json::json!({
        "name": "stalint",
        "title": "Structural Linter",
        "description": "Check files for structural violations \
                        (trailing whitespace, tabs/spaces, line endings, file size). \
                        Read-only - returns diagnostics without modifying files. \
                        To fix: use dictator tool (default mode handles supreme/* rules). \
                        Use mode=supremecourt only for external linter auto-fix \
                        (rubocop -a, ruff --fix, eslint --fix, clippy --fix).",
        "annotations": {
            "title": "Structural Linter",
            "readOnlyHint": true,
            "openWorldHint": false
        },
        "inputSchema": {
            "type": "object",
            "properties": {
                "paths": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "File or directory paths to check"
                },
                "limit": {
                    "type": "integer",
                    "description": "Max violations to return (default 10)"
                },
                "cursor": {
                    "type": "string",
                    "description": "Pagination cursor from previous response"
                }
            },
            "required": ["paths"]
        }
    })];

    // Only include write tools if conditions are met:
    // 1. Can write (not in read-only sandbox)
    // 2. Inside a git repository (safety check)
    let in_git_repo = is_git_repo();
    if can_write && in_git_repo {
        // Check which configured linters are available
        let mut state = watcher_state.lock().unwrap();
        state.ensure_config_loaded();
        let has_supreme = state.config.as_ref().is_some_and(|cfg| {
            cfg.decree.values().any(|decree| {
                decree
                    .linter
                    .as_ref()
                    .is_some_and(|linter| command_available(&linter.command))
            })
        });
        drop(state);

        // Build mode enum based on available external linters
        let modes: Vec<&str> = if has_supreme {
            vec!["kimjongrails", "supremecourt"]
        } else {
            vec!["kimjongrails"]
        };

        tool_list.push(serde_json::json!({
            "name": "dictator",
            "title": "Auto-Fixer",
            "description": "Auto-fix structural issues (whitespace, newlines, line endings). \
                           Requires git repository.",
            "annotations": {
                "title": "Auto-Fixer",
                "readOnlyHint": false,
                "destructiveHint": true,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "File or directory paths to fix"
                    },
                    "mode": {
                        "type": "string",
                        "enum": modes,
                        "description": "kimjongrails (default): basic fixes. \
                                       supremecourt: basic + external linter auto-fix"
                    }
                },
                "required": ["paths"]
            }
        }));
    }

    tool_list.push(watch_tool);

    let tools = serde_json::json!({ "tools": tool_list });

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(tools),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::state::DictateConfig;

    #[test]
    fn test_handle_list_tools_not_watching() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        {
            let mut s = state.lock().unwrap();
            s.config = Some(DictateConfig::default());
        }
        let id = serde_json::json!(1);

        let response = handle_list_tools(id, state);

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();

        // Always has stalint and stalint_watch
        // dictator only if in git repo
        assert!(tools.len() >= 2);

        let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(tool_names.contains(&"stalint"));
        assert!(tool_names.contains(&"stalint_watch"));
        assert!(!tool_names.contains(&"stalint_unwatch"));
        // dictator present only if .git exists
        if is_git_repo() {
            assert!(tool_names.contains(&"dictator"));
        }
    }

    #[test]
    fn test_handle_list_tools_watching() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        {
            let mut s = state.lock().unwrap();
            s.is_watching = true;
            s.config = Some(DictateConfig::default());
        }
        let id = serde_json::json!(1);

        let response = handle_list_tools(id, state);

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();

        let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(tool_names.contains(&"stalint_unwatch"));
        assert!(!tool_names.contains(&"stalint_watch"));
    }

    #[test]
    fn test_handle_list_tools_annotations() {
        let state = Arc::new(Mutex::new(ServerState::default()));
        {
            let mut s = state.lock().unwrap();
            s.config = Some(DictateConfig::default());
        }
        let id = serde_json::json!(1);

        let response = handle_list_tools(id, state);

        let result = response.result.unwrap();
        let tools = result["tools"].as_array().unwrap();

        // stalint should be read-only
        let stalint = tools.iter().find(|t| t["name"] == "stalint").unwrap();
        assert_eq!(stalint["annotations"]["readOnlyHint"], true);

        // dictator annotations (only if in git repo)
        if is_git_repo() {
            let dictator = tools.iter().find(|t| t["name"] == "dictator").unwrap();
            assert_eq!(dictator["annotations"]["readOnlyHint"], false);
            assert_eq!(dictator["annotations"]["destructiveHint"], true);
            assert_eq!(dictator["annotations"]["idempotentHint"], true);
        }
    }
}
