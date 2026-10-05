use super::*;

const CATALOG: &str = "- `dictator://mood`: The Dictator's current disposition";

#[test]
fn occupied_without_catalog_keeps_static_instructions() {
    assert_eq!(compose(true, None), None);
}

#[test]
fn unoccupied_points_at_occupy() {
    let text = compose(false, None).expect("unoccupied instructions");
    assert!(text.contains("`occupy`"));
    assert!(!text.contains("Resources"));
}

#[test]
fn catalog_is_appended_to_base() {
    let text = compose(true, Some(CATALOG.to_string())).expect("instructions");
    assert!(text.starts_with(BASE));
    assert!(text.ends_with(CATALOG));
}

#[test]
fn catalog_is_appended_while_unoccupied() {
    let text = compose(false, Some(CATALOG.to_string())).expect("instructions");
    assert!(text.contains("`occupy`"));
    assert!(text.ends_with(CATALOG));
}

#[test]
fn claude_code_hides_resources_from_its_model() {
    assert!(!model_sees_resources(&Implementation::new(
        "claude-code",
        "2.1.0"
    )));
    assert!(model_sees_resources(&Implementation::new(
        "codex-mcp-client",
        "0.40.0"
    )));
}

#[test]
fn resource_error_specs_parse() {
    for spec in HIDES_RESOURCE_ERRORS {
        assert!(spec.parse::<ClientMatcher>().is_ok(), "bad spec {spec}");
    }
}
