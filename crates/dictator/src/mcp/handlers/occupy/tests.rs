use super::*;

#[test]
fn issue_tracker_routes_by_client() {
    assert_eq!(
        issue_tracker(Some("claude-code")),
        "https://github.com/anthropics/claude-code/issues"
    );
    assert_eq!(
        issue_tracker(Some("codex-mcp-client")),
        "https://github.com/openai/codex/issues"
    );
    assert_eq!(
        issue_tracker(Some("zed")),
        "https://github.com/seuros/dictator/issues"
    );
    assert_eq!(
        issue_tracker(None),
        "https://github.com/seuros/dictator/issues"
    );
}
