use super::*;

fn state() -> Arc<Mutex<ServerState>> {
    Arc::new(Mutex::new(ServerState::default()))
}

#[test]
fn list_resources_reflects_config_state() {
    // Without a loaded config there is nothing to list.
    let response = handle_list_resources(serde_json::json!(1), state());
    assert!(response.error.is_none());
    let result = response.result.unwrap();
    assert_eq!(result["resources"].as_array().unwrap().len(), 0);

    // With a config, both resources appear in order.
    let loaded = state();
    loaded.lock().unwrap().config = Some(super::super::state::DictateConfig::default());
    let response = handle_list_resources(serde_json::json!(1), loaded);
    assert!(response.error.is_none());
    let result = response.result.unwrap();
    let resources = result["resources"].as_array().unwrap();
    assert_eq!(resources.len(), 2);
    assert_eq!(resources[0]["uri"], CONFIG_URI);
    assert_eq!(resources[1]["uri"], CENSUS_URI);
}

#[test]
fn read_resource_handles_valid_and_invalid_requests() {
    // Missing params and missing uri are both invalid params.
    for params in [None, Some(serde_json::json!({}))] {
        let response = handle_read_resource(serde_json::json!(1), params, state());
        assert_eq!(response.error.unwrap().code, -32602);
    }

    // Unknown URI is resource-not-found.
    let params = Some(serde_json::json!({"uri": "dictator://unknown"}));
    let response = handle_read_resource(serde_json::json!(1), params, state());
    assert_eq!(response.error.unwrap().code, -32002);

    // Both known URIs resolve to a single content entry echoing the URI.
    for uri in [CONFIG_URI, CENSUS_URI] {
        let params = Some(serde_json::json!({ "uri": uri }));
        let response = handle_read_resource(serde_json::json!(1), params, state());
        assert!(response.error.is_none(), "{uri} should read");
        let result = response.result.unwrap();
        let contents = result["contents"].as_array().unwrap();
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0]["uri"], uri);
    }
}
