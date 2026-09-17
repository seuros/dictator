//! Integration tests for diff-scoped linting against a real git repository.

use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

const BIN: &str = env!("CARGO_BIN_EXE_dictator");

/// Five functions, each pair of lines carrying trailing whitespace.
const LEGACY: &str = "def legacy_0():   \n    pass   \ndef legacy_1():   \n    pass   \n\
                      def legacy_2():   \n    pass   \n";

fn git(repo: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .args(args)
        .current_dir(repo)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .status()?;
    assert!(status.success(), "git {args:?} failed");
    Ok(())
}

fn init_repo() -> Result<TempDir> {
    let dir = TempDir::new()?;
    let path = dir.path();
    git(path, &["init", "-q", "--template="])?;
    git(path, &["config", "user.email", "dictator@example.com"])?;
    git(path, &["config", "user.name", "The Dictator"])?;
    fs::write(path.join("legacy.py"), LEGACY)?;
    git(path, &["add", "-A"])?;
    git(path, &["commit", "-qm", "the legacy timeline"])?;
    Ok(dir)
}

/// Insert a line with its own violation between two legacy lines.
fn touch_one_line(repo: &Path) -> Result<()> {
    let mut lines: Vec<String> = LEGACY.lines().map(ToString::to_string).collect();
    lines.insert(2, "NEW_GLOBAL = 1   ".to_string());
    fs::write(repo.join("legacy.py"), lines.join("\n") + "\n")?;
    Ok(())
}

fn dictator(repo: &Path, args: &[&str]) -> Result<Output> {
    Ok(Command::new(BIN).args(args).current_dir(repo).output()?)
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn diff_scope_reports_only_the_touched_line() -> Result<()> {
    let dir = init_repo()?;
    touch_one_line(dir.path())?;

    let full = dictator(dir.path(), &["lint", "."])?;
    let scoped = dictator(dir.path(), &["lint", "--diff", "HEAD", "."])?;

    let full_hits = stdout(&full).matches("trailing-whitespace").count();
    let scoped_hits = stdout(&scoped).matches("trailing-whitespace").count();

    assert_eq!(
        full_hits, 7,
        "unscoped run should see every legacy violation"
    );
    assert_eq!(scoped_hits, 1, "scoped run should see only the new line");
    assert!(stdout(&scoped).contains("legacy.py:3:"));
    assert!(stderr(&scoped).contains("6 legacy violation(s) withheld"));
    Ok(())
}

#[test]
fn diff_scope_fix_leaves_untouched_lines_alone() -> Result<()> {
    let dir = init_repo()?;
    touch_one_line(dir.path())?;

    dictator(dir.path(), &["lint", "--diff", "HEAD", "--fix", "."])?;

    let after = fs::read_to_string(dir.path().join("legacy.py"))?;
    let lines: Vec<&str> = after.lines().collect();
    assert_eq!(lines[2], "NEW_GLOBAL = 1", "the touched line is fixed");
    assert_eq!(lines[0], "def legacy_0():   ", "legacy lines are untouched");
    assert_eq!(lines[1], "    pass   ", "legacy lines are untouched");
    Ok(())
}

#[test]
fn unscoped_fix_still_rewrites_the_whole_file() -> Result<()> {
    let dir = init_repo()?;
    touch_one_line(dir.path())?;

    dictator(dir.path(), &["lint", "--fix", "."])?;

    let after = fs::read_to_string(dir.path().join("legacy.py"))?;
    assert!(
        !after.lines().any(|l| l.ends_with(' ')),
        "every line should be clean without a diff selector"
    );
    Ok(())
}

#[test]
fn staged_scope_sees_only_the_index() -> Result<()> {
    let dir = init_repo()?;
    touch_one_line(dir.path())?;

    let unstaged = dictator(dir.path(), &["lint", "--staged", "."])?;
    assert!(stderr(&unstaged).contains("No changed files in range"));

    git(dir.path(), &["add", "legacy.py"])?;
    let staged = dictator(dir.path(), &["lint", "--staged", "."])?;
    assert_eq!(stdout(&staged).matches("trailing-whitespace").count(), 1);
    Ok(())
}

#[test]
fn untracked_files_are_entirely_in_scope() -> Result<()> {
    let dir = init_repo()?;
    fs::write(dir.path().join("brand_new.py"), "x = 1   \ny = 2   \n")?;

    let scoped = dictator(dir.path(), &["lint", "--diff", "HEAD", "."])?;
    assert_eq!(
        stdout(&scoped).matches("trailing-whitespace").count(),
        2,
        "git diff omits untracked files, but every line of a new file is new"
    );

    // Not in the index, so --staged must still ignore it.
    let staged = dictator(dir.path(), &["lint", "--staged", "."])?;
    assert!(stderr(&staged).contains("No changed files in range"));
    Ok(())
}

#[test]
fn ignored_files_stay_out_of_scope() -> Result<()> {
    let dir = init_repo()?;
    fs::write(dir.path().join(".gitignore"), "generated.py\n")?;
    fs::write(dir.path().join("generated.py"), "x = 1   \n")?;

    let scoped = dictator(dir.path(), &["lint", "--diff", "HEAD", "."])?;
    assert!(
        !stdout(&scoped).contains("generated.py"),
        "gitignored files are not the author's work"
    );
    Ok(())
}

#[test]
fn file_scope_rules_survive_the_filter() -> Result<()> {
    let dir = init_repo()?;
    // Strip the final newline without touching any existing line's content.
    let trimmed = LEGACY.trim_end_matches('\n').to_string();
    fs::write(dir.path().join("legacy.py"), &trimmed)?;

    let scoped = dictator(dir.path(), &["lint", "--diff", "HEAD", "."])?;
    assert!(
        stdout(&scoped).contains("missing-final-newline"),
        "a whole-file rule must report even though its span is outside the hunk"
    );
    Ok(())
}

#[test]
fn github_format_emits_workflow_commands() -> Result<()> {
    let dir = init_repo()?;
    touch_one_line(dir.path())?;

    let out = dictator(
        dir.path(),
        &["lint", "--diff", "HEAD", "--format", "github", "."],
    )?;
    let text = stdout(&out);
    assert!(text.contains("::warning file=legacy.py,line=3,col=15,"));
    assert!(text.contains("title=supreme/trailing-whitespace::trailing whitespace"));
    assert!(!text.contains("file=./"), "paths must be repo-relative");
    Ok(())
}

#[test]
fn diff_and_staged_are_mutually_exclusive() -> Result<()> {
    let dir = init_repo()?;
    let out = dictator(dir.path(), &["lint", "--diff", "HEAD", "--staged", "."])?;
    assert!(stderr(&out).contains("cannot be combined"));
    Ok(())
}

#[test]
fn diff_outside_a_repository_explains_itself() -> Result<()> {
    let dir = TempDir::new()?;
    fs::write(dir.path().join("a.py"), "x = 1   \n")?;

    let out = dictator(dir.path(), &["lint", "--diff", "HEAD", "."])?;
    assert!(stderr(&out).contains("not inside a git repository"));
    Ok(())
}
