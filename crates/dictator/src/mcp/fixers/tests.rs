use super::*;
use std::path::Path;

fn fix_workspace(dir: &Path) -> String {
    let state = Arc::new(Mutex::new(ServerState::default()));
    let args = serde_json::json!({
        "paths": [dir.to_string_lossy()],
        "workspace": dir.to_string_lossy(),
    });
    let response = handle_kimjongrails(serde_json::json!(1), Some(args), state);
    response.result.expect("result")["content"][0]["text"]
        .as_str()
        .expect("text")
        .to_string()
}

#[test]
fn applies_the_workspace_config_not_a_blanket_rewrite() {
    let dir = tempfile::tempdir().unwrap();
    let write = |name: &str, body: &str| std::fs::write(dir.path().join(name), body).unwrap();
    write(
        ".dictate.toml",
        "[decree.supreme]\nline_endings = \"crlf\"\n\n\
         [decree.supreme.ignore.trailing-whitespace]\nextensions = [\"md\"]\n\n\
         [decree.ruby]\n",
    );
    write("notes.md", "hard  \r\nbreak\r\n");
    write("app.rb", "#bad  \r\nx = 1\n");
    write(".env", "SECRET=1  \n");

    let summary = fix_workspace(dir.path());
    let read = |name: &str| std::fs::read_to_string(dir.path().join(name)).unwrap();

    assert_eq!(
        read("notes.md"),
        "hard  \r\nbreak\r\n",
        "ignored rule stays"
    );
    assert_eq!(read("app.rb"), "# bad\r\nx = 1\r\n", "ruby decree + crlf");
    assert_eq!(read(".env"), "SECRET=1  \n", "classified is never touched");
    assert!(summary.contains("ruby/comment-space"), "{summary}");
}

#[test]
fn reports_nothing_to_fix_on_a_compliant_tree() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(".dictate.toml"), "[decree.supreme]\n").unwrap();
    std::fs::write(dir.path().join("ok.txt"), "fine\n").unwrap();

    assert_eq!(fix_workspace(dir.path()), "No fixes needed");
}
