use super::*;
use crate::mcp::utils::base64_encode;

#[test]
fn stalint_handler_covers_defaults_paths_and_pagination() {
    let state = || Arc::new(Mutex::new(ServerState::default()));

    // No arguments defaults to cwd and succeeds.
    let response = handle_stalint(serde_json::json!(1), None, state());
    assert!(response.error.is_none());
    assert!(response.result.unwrap()["structuredContent"]["total"].is_number());

    // Nonexistent path lints nothing, and a limit parses fine alongside it.
    let args = Some(serde_json::json!({
        "paths": ["/nonexistent/path/that/does/not/exist"],
        "limit": 5
    }));
    let response = handle_stalint(serde_json::json!(1), args, state());
    assert!(response.error.is_none());
    let result = response.result.unwrap();
    assert_eq!(result["structuredContent"]["total"], 0);
    assert_eq!(result["structuredContent"]["returned"], 0);

    // Paths from a first call are stored and reused by a cursor-only follow-up.
    let shared = state();
    let args = Some(serde_json::json!({"paths": ["/nonexistent"]}));
    let response = handle_stalint(serde_json::json!(1), args, Arc::clone(&shared));
    assert!(response.error.is_none());
    let args = Some(serde_json::json!({"cursor": base64_encode(b"0")}));
    let response = handle_stalint(serde_json::json!(2), args, shared);
    assert!(response.error.is_none());

    // A cursor without stored paths is an invalid-params error.
    let args = Some(serde_json::json!({"cursor": base64_encode(b"10")}));
    let response = handle_stalint(serde_json::json!(1), args, state());
    let error = response.error.unwrap();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("no paths stored"));
}
