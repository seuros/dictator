use super::{GitScope, files_fingerprint, git_changed_files, make_snippet};
use dictator_decree_abi::Span;
use std::process::Command;

#[test]
fn snippet_survives_span_inside_multibyte_char() {
    let source = "héllo wörld\n";
    for start in 0..=source.len() {
        let _ = make_snippet(source, &Span { start, end: start }, 160);
    }
    let snip = make_snippet(source, &Span { start: 2, end: 2 }, 160);
    assert_eq!(snip, "héllo wörld");
}

fn git(dir: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed in {dir:?}");
}

#[test]
fn git_uncommitted_files_walkthrough() {
    // Outside a repo there is no scope at all.
    let plain = tempfile::tempdir().unwrap();
    assert!(git_changed_files(plain.path(), GitScope::Uncommitted).is_none());

    let tmp = tempfile::tempdir().unwrap();
    git(tmp.path(), &["init", "-q"]);
    git(tmp.path(), &["config", "user.email", "test@example.com"]);
    git(tmp.path(), &["config", "user.name", "Test"]);

    // Clean repo returns an empty scope.
    std::fs::write(tmp.path().join("committed.txt"), "v1").unwrap();
    git(tmp.path(), &["add", "."]);
    git(tmp.path(), &["commit", "-q", "-m", "init"]);
    let files =
        git_changed_files(tmp.path(), GitScope::Uncommitted).expect("should detect git repo");
    assert!(files.is_empty());

    // Modified and untracked files are both picked up.
    std::fs::write(tmp.path().join("committed.txt"), "v2").unwrap();
    std::fs::write(tmp.path().join("new.txt"), "new").unwrap();
    let files =
        git_changed_files(tmp.path(), GitScope::Uncommitted).expect("should detect git repo");
    let names: Vec<_> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();
    assert!(names.contains(&"committed.txt"));
    assert!(names.contains(&"new.txt"));

    // Staged scope sees only the index: stage one file, edit another unstaged.
    git(tmp.path(), &["add", "new.txt"]);
    let staged = git_changed_files(tmp.path(), GitScope::Staged).expect("should detect git repo");
    let staged_names: Vec<_> = staged
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap())
        .collect();
    assert_eq!(staged_names, ["new.txt"]);

    // Fingerprint changes when a file's content changes.
    let all = git_changed_files(tmp.path(), GitScope::Uncommitted).unwrap();
    let before = files_fingerprint(&all);
    std::fs::write(tmp.path().join("committed.txt"), "v3-longer").unwrap();
    assert_ne!(before, files_fingerprint(&all));

    // Deleted files drop back out of scope.
    git(tmp.path(), &["add", "."]);
    git(tmp.path(), &["commit", "-q", "-m", "second"]);
    std::fs::remove_file(tmp.path().join("new.txt")).unwrap();
    let files =
        git_changed_files(tmp.path(), GitScope::Uncommitted).expect("should detect git repo");
    assert!(files.is_empty());
}
