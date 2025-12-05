//! File collection, path utilities, and helper functions.

use std::fs::OpenOptions;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Get per-worktree cache directory: `.dictator/cache` under the current working dir.
/// Falls back to XDG cache if cwd cannot be determined.
pub fn get_cache_dir() -> std::path::PathBuf {
    if let Ok(cwd) = std::env::current_dir() {
        let cache = cwd.join(".dictator").join("cache");
        let _ = std::fs::create_dir_all(&cache);
        // Restrict to user only; ignore errors quietly.
        #[cfg(unix)]
        let _ = std::fs::set_permissions(&cache, std::fs::Permissions::from_mode(0o700));
        return cache;
    }

    // Fallback to XDG_CACHE_HOME / $HOME/.cache when cwd unavailable
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

/// Check if current directory is a git repository
pub fn is_git_repo() -> bool {
    let cwd = std::env::current_dir().unwrap_or_default();
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

/// Collect files recursively from a path
pub fn collect_files(path: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if path.is_file() {
        files.push(path.to_path_buf());
    } else if path.is_dir()
        && let Ok(entries) = std::fs::read_dir(path)
    {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                files.push(p);
            } else if p.is_dir() {
                files.extend(collect_files(&p));
            }
        }
    }
    files
}

/// Convert byte offset to line and column numbers
pub fn byte_to_line_col(src: &str, byte_idx: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in src.char_indices() {
        if i == byte_idx {
            return (line, col);
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Check if a path is within the current working directory (security boundary)
pub fn is_within_cwd(path: &std::path::Path, cwd: &std::path::Path) -> bool {
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };

    // Try canonicalize first (handles symlinks and ..)
    if let (Ok(resolved_canon), Ok(cwd_canon)) = (resolved.canonicalize(), cwd.canonicalize()) {
        return resolved_canon.starts_with(&cwd_canon);
    }

    // For non-existent paths, do basic check:
    // - Absolute paths outside cwd are rejected
    // - Relative paths with .. that escape are rejected
    if path.is_absolute() {
        if let Ok(cwd_canon) = cwd.canonicalize() {
            return resolved.starts_with(&cwd_canon);
        }
        return false;
    }

    // Relative path - check it doesn't start with .. that would escape
    let mut depth: i32 = 0;
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => depth -= 1,
            std::path::Component::Normal(_) => depth += 1,
            _ => {}
        }
        if depth < 0 {
            return false; // Escapes cwd with ..
        }
    }
    true // Relative path stays within cwd
}

/// Base64 encode bytes
pub fn base64_encode(data: &[u8]) -> String {
    use base64::{Engine, engine::general_purpose::STANDARD};
    STANDARD.encode(data)
}

/// Base64 decode string
pub fn base64_decode(s: &str) -> Vec<u8> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    STANDARD.decode(s).unwrap_or_default()
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

#[cfg(test)]
mod tests {
    use super::*;

    // ========== is_within_cwd tests ==========

    #[test]
    fn test_is_within_cwd_relative_path() {
        let cwd = std::env::current_dir().unwrap();
        assert!(is_within_cwd(std::path::Path::new("foo"), &cwd));
        assert!(is_within_cwd(std::path::Path::new("foo/bar"), &cwd));
        assert!(is_within_cwd(std::path::Path::new("./foo"), &cwd));
    }

    #[test]
    fn test_is_within_cwd_parent_escape() {
        let cwd = std::env::current_dir().unwrap();
        assert!(!is_within_cwd(std::path::Path::new("../foo"), &cwd));
        assert!(!is_within_cwd(std::path::Path::new("foo/../../bar"), &cwd));
        assert!(!is_within_cwd(std::path::Path::new(".."), &cwd));
    }

    #[test]
    fn test_is_within_cwd_absolute_outside() {
        let cwd = std::env::current_dir().unwrap();
        assert!(!is_within_cwd(std::path::Path::new("/tmp"), &cwd));
        assert!(!is_within_cwd(std::path::Path::new("/etc/passwd"), &cwd));
        assert!(!is_within_cwd(std::path::Path::new("/home"), &cwd));
    }

    // ========== Cursor pagination tests ==========

    #[test]
    fn test_base64_encode_decode_roundtrip() {
        let original = "42";
        let encoded = base64_encode(original.as_bytes());
        let decoded = String::from_utf8(base64_decode(&encoded)).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_base64_encode_decode_offset() {
        let offsets = [0, 10, 100, 12345];
        for offset in offsets {
            let encoded = base64_encode(offset.to_string().as_bytes());
            let decoded: usize = String::from_utf8(base64_decode(&encoded))
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(offset, decoded);
        }
    }

    #[test]
    fn test_base64_decode_invalid() {
        // Invalid base64 should return empty vec
        let result = base64_decode("!!!invalid!!!");
        assert!(result.is_empty());
    }

    // ========== byte_to_line_col tests ==========

    #[test]
    fn test_byte_to_line_col_start() {
        let src = "hello\nworld\n";
        let (line, col) = byte_to_line_col(src, 0);
        assert_eq!((line, col), (1, 1));
    }

    #[test]
    fn test_byte_to_line_col_middle_first_line() {
        let src = "hello\nworld\n";
        let (line, col) = byte_to_line_col(src, 2);
        assert_eq!((line, col), (1, 3)); // 'l' at position 2
    }

    #[test]
    fn test_byte_to_line_col_second_line() {
        let src = "hello\nworld\n";
        let (line, col) = byte_to_line_col(src, 6);
        assert_eq!((line, col), (2, 1)); // 'w' at start of line 2
    }

    #[test]
    fn test_byte_to_line_col_end() {
        let src = "hello\nworld\n";
        let (line, col) = byte_to_line_col(src, 11);
        assert_eq!((line, col), (2, 6)); // newline at end of 'world'
    }

    #[test]
    fn test_byte_to_line_col_beyond_end() {
        let src = "hi";
        let (line, col) = byte_to_line_col(src, 100);
        // Should return last position
        assert_eq!((line, col), (1, 3));
    }
}
