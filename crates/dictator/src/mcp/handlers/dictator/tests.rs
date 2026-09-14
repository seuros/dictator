use super::*;

#[test]
fn dictator_handler_validates_arguments_and_scope() {
    let state = || Arc::new(Mutex::new(ServerState::default()));

    // Missing arguments is invalid params.
    let response = handle_dictator(serde_json::json!(1), None, state());
    assert_eq!(response.error.unwrap().code, -32602);

    // Unknown mode is rejected (relative path passes the cwd security check).
    let args = Some(serde_json::json!({"paths": ["sandbox"], "mode": "unknown_mode"}));
    let response = handle_dictator(serde_json::json!(1), args, state());
    let error = response.error.unwrap();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("Unknown mode"));

    // Default mode (kimjongrails) parses even for a nonexistent relative path.
    let args = Some(serde_json::json!({"paths": ["nonexistent_but_within_cwd"]}));
    let response = handle_dictator(serde_json::json!(1), args, state());
    assert!(response.error.is_none());

    // Absolute paths outside cwd are refused.
    let args = Some(serde_json::json!({"paths": ["/tmp", "/etc"]}));
    let response = handle_dictator(serde_json::json!(1), args, state());
    let error = response.error.unwrap();
    assert_eq!(error.code, -32602);
    assert!(error.message.contains("Security"));
    assert!(error.message.contains("only operates within cwd"));
}
