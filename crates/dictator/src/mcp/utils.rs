//! File collection, path utilities, and helper functions.

use mcp_host::protocol::types::{JsonRpcError, JsonRpcResponse};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

/// Get per-worktree cache directory: `.dictator/cache` under the current working dir.
/// Only uses local `.dictator/` if `.dictate.toml` config exists (repo is occupied).
/// Falls back to XDG cache otherwise to avoid polluting unoccupied repos.
pub fn get_cache_dir() -> std::path::PathBuf {
    if let Ok(cwd) = std::env::current_dir() {
        let config_file = cwd.join(".dictate.toml");
        // Only use local cache if config exists (repo is occupied)
        if config_file.exists() {
            let cache = cwd.join(".dictator").join("cache");
            let _ = std::fs::create_dir_all(&cache);
            // Restrict to user only; ignore errors quietly.
            let _ = std::fs::set_permissions(&cache, std::fs::Permissions::from_mode(0o700));
            return cache;
        }
    }

    // Fallback to XDG_CACHE_HOME / $HOME/.cache when not occupied
    let cache_dir = std::env::var("XDG_CACHE_HOME")
        .ok()
        .or_else(|| std::env::var("HOME").ok().map(|h| format!("{h}/.cache")))
        .unwrap_or_else(|| "/tmp".to_string());

    let dictator_cache = std::path::Path::new(&cache_dir).join("dictator");
    let _ = std::fs::create_dir_all(&dictator_cache);
    dictator_cache
}

/// Get log file path for a given filename
pub fn get_log_path(filename: &str) -> std::path::PathBuf {
    get_cache_dir().join(filename)
}

/// Simple file logger
pub fn log_to_file(msg: &str) {
    let log_file = get_log_path("mcp.log");
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_file) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let _ = writeln!(
            file,
            "[{}.{:03}] {}",
            now.as_secs(),
            now.subsec_millis(),
            msg
        );
    }
}

/// Current working directory, falling back to an empty path on failure.
#[must_use]
pub fn current_dir_or_default() -> std::path::PathBuf {
    std::env::current_dir().unwrap_or_default()
}

/// Standard JSON-RPC response for invalid tool arguments.
#[must_use]
pub fn invalid_arguments_response(id: serde_json::Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id: Some(id),
        result: None,
        error: Some(JsonRpcError {
            code: -32602,
            message: "Missing or invalid arguments".to_string(),
            data: None,
        }),
    }
}

/// Parse optional JSON-RPC arguments into a typed struct.
pub fn parse_arguments<T: DeserializeOwned>(
    id: &serde_json::Value,
    arguments: Option<serde_json::Value>,
) -> Result<T, Box<JsonRpcResponse>> {
    arguments
        .and_then(|a| serde_json::from_value(a).ok())
        .ok_or_else(|| Box::new(invalid_arguments_response(id.clone())))
}

/// Check if current directory is a git repository
#[allow(dead_code)]
pub fn is_git_repo() -> bool {
    let cwd = current_dir_or_default();
    cwd.join(".git").exists()
}

/// Check if a command is available in PATH
pub fn command_available(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

// Re-export mcp-host utilities
pub use mcp_host::utils::{base64_decode, base64_encode, byte_to_line_col, collect_files};

/// List files with uncommitted changes (staged, unstaged, or untracked) in the git
/// repository containing `dir`.
///
/// Returns `None` when `dir` is not inside a git repo (or `git` isn't available), so
/// callers can fall back to scanning the whole directory. Returns `Some(vec![])` for a
/// clean repo — that's a meaningful "nothing to do" result, distinct from "not a repo".
pub fn git_uncommitted_files(dir: &std::path::Path) -> Option<Vec<std::path::PathBuf>> {
    let toplevel_out = std::process::Command::new("git")
        .args(["-C", &dir.to_string_lossy(), "rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !toplevel_out.status.success() {
        return None;
    }
    let toplevel = String::from_utf8_lossy(&toplevel_out.stdout)
        .trim()
        .to_string();

    // -z: NUL-separated, unquoted paths. --no-renames: report renames as plain
    // delete+add so each entry is a single path, not "old -> new".
    let status_out = std::process::Command::new("git")
        .args([
            "-C",
            &toplevel,
            "status",
            "--porcelain=v1",
            "--no-renames",
            "-z",
        ])
        .output()
        .ok()?;
    if !status_out.status.success() {
        return None;
    }

    let root = std::path::Path::new(&toplevel);
    let files = status_out
        .stdout
        .split(|&b| b == 0)
        .filter(|entry| entry.len() > 3)
        .map(|entry| root.join(String::from_utf8_lossy(&entry[3..]).as_ref()))
        // Drop deleted files — nothing left to lint.
        .filter(|p| p.is_file())
        .collect();

    Some(files)
}

/// Check if a path is within the current working directory (security boundary)
/// Wrapper around mcp-host's `is_safe_path` for dictator-specific naming
pub fn is_within_cwd(path: &std::path::Path, cwd: &std::path::Path) -> bool {
    mcp_host::utils::is_safe_path(path, cwd)
}

/// Partition paths into allowed and rejected sets based on the cwd security boundary.
#[must_use]
pub fn partition_paths_within_cwd(
    paths: &[String],
    cwd: &std::path::Path,
) -> (Vec<String>, Vec<String>) {
    let mut allowed = Vec::new();
    let mut rejected = Vec::new();

    for path in paths {
        let candidate = std::path::Path::new(path);
        if is_within_cwd(candidate, cwd) {
            allowed.push(path.clone());
        } else {
            rejected.push(path.clone());
        }
    }

    (allowed, rejected)
}

/// Validate that every path lies within `cwd`, returning the allowed paths.
///
/// On any rejected path, or when nothing valid remains, returns the appropriate
/// JSON-RPC error response (boxed) for the handler to return directly. `tool`
/// names the tool in the security message.
pub fn allowed_paths_within_cwd(
    id: &serde_json::Value,
    paths: &[String],
    tool: &str,
) -> Result<Vec<String>, Box<JsonRpcResponse>> {
    let cwd = current_dir_or_default();
    let (allowed, rejected) = partition_paths_within_cwd(paths, &cwd);

    if !rejected.is_empty() {
        return Err(Box::new(JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: Some(id.clone()),
            result: None,
            error: Some(JsonRpcError {
                code: -32602,
                message: format!(
                    "Security: {tool} only operates within cwd ({}). Rejected paths: {}",
                    cwd.display(),
                    rejected.join(", ")
                ),
                data: None,
            }),
        }));
    }

    if allowed.is_empty() {
        return Err(Box::new(JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: Some(id.clone()),
            result: None,
            error: Some(JsonRpcError {
                code: -32602,
                message: "No valid paths provided".to_string(),
                data: None,
            }),
        }));
    }

    Ok(allowed)
}

/// Serialize a value to compact JSON, falling back to an empty string on failure.
#[must_use]
pub fn to_json_string<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

/// Serialize a value to pretty JSON, falling back to an empty string on failure.
#[must_use]
pub fn to_json_string_pretty<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_default()
}

/// Build a sanitized single-line snippet around the diagnostic span.
pub fn make_snippet(source: &str, span: &dictator_decree_abi::Span, max_len: usize) -> String {
    if source.is_empty() {
        return String::new();
    }

    let start = span.start.min(source.len());

    // Find line bounds containing the span start.
    let line_start = source[..start].rfind('\n').map_or(0, |idx| idx + 1);
    let line_end = source[start..]
        .find('\n')
        .map_or_else(|| source.len(), |off| start + off);

    let line = &source[line_start..line_end];

    // Sanitize control characters (except tab) to spaces and trim trailing whitespace.
    let mut cleaned: String = line
        .chars()
        .map(|c| if c.is_control() && c != '\t' { ' ' } else { c })
        .collect();
    cleaned.truncate(cleaned.trim_end().len());

    if cleaned.len() > max_len {
        let mut out = cleaned
            .chars()
            .take(max_len.saturating_sub(1))
            .collect::<String>();
        out.push('…');
        out
    } else {
        cleaned
    }
}

// Tests for moved functions are now in mcp-host crate

#[cfg(test)]
mod git_scope_tests {
    use super::git_uncommitted_files;
    use std::process::Command;

    fn git(dir: &std::path::Path, args: &[&str]) {
        let status = Command::new("git")
            .current_dir(dir)
            .args(args)
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed in {dir:?}");
    }

    fn init_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        git(tmp.path(), &["init", "-q"]);
        git(tmp.path(), &["config", "user.email", "test@example.com"]);
        git(tmp.path(), &["config", "user.name", "Test"]);
        tmp
    }

    #[test]
    fn not_a_git_repo_returns_none() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(git_uncommitted_files(tmp.path()).is_none());
    }

    #[test]
    fn clean_repo_returns_empty() {
        let tmp = init_repo();
        std::fs::write(tmp.path().join("a.txt"), "hello").unwrap();
        git(tmp.path(), &["add", "."]);
        git(tmp.path(), &["commit", "-q", "-m", "init"]);

        let files = git_uncommitted_files(tmp.path()).expect("should detect git repo");
        assert!(files.is_empty());
    }

    #[test]
    fn detects_untracked_and_modified_files() {
        let tmp = init_repo();
        std::fs::write(tmp.path().join("committed.txt"), "v1").unwrap();
        git(tmp.path(), &["add", "."]);
        git(tmp.path(), &["commit", "-q", "-m", "init"]);

        std::fs::write(tmp.path().join("committed.txt"), "v2").unwrap();
        std::fs::write(tmp.path().join("new.txt"), "new").unwrap();

        let files = git_uncommitted_files(tmp.path()).expect("should detect git repo");
        let names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert!(names.contains(&"committed.txt"));
        assert!(names.contains(&"new.txt"));
    }

    #[test]
    fn deleted_files_are_excluded() {
        let tmp = init_repo();
        std::fs::write(tmp.path().join("gone.txt"), "bye").unwrap();
        git(tmp.path(), &["add", "."]);
        git(tmp.path(), &["commit", "-q", "-m", "init"]);
        std::fs::remove_file(tmp.path().join("gone.txt")).unwrap();

        let files = git_uncommitted_files(tmp.path()).expect("should detect git repo");
        assert!(files.is_empty());
    }
}
