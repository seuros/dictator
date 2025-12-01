//! Dictate command implementation - structural issue fixing

use anyhow::Result;
use std::fs;

use crate::cli::DictateArgs;
use crate::files::collect_all_files;

pub fn run_dictate(args: DictateArgs) -> Result<()> {
    let files = collect_all_files(&args.paths)?;
    if files.is_empty() {
        eprintln!("No files found");
        return Ok(());
    }

    let mut fixed_count = 0;
    let mut file_count = 0;

    for path in files {
        let original = fs::read_to_string(&path)?;
        let fixed = fix_structural_issues(&original);

        if fixed != original {
            fs::write(&path, &fixed)?;
            println!("Fixed: {path}");
            fixed_count += 1;
        }
        file_count += 1;
    }

    if fixed_count > 0 {
        println!("Fixed {fixed_count} file(s) out of {file_count} checked.");
    } else {
        println!("All {file_count} file(s) already compliant.");
    }

    Ok(())
}

/// Fix structural issues: trailing whitespace, line endings, final newline, blank line whitespace
pub fn fix_structural_issues(content: &str) -> String {
    let mut result = String::with_capacity(content.len());

    // Normalize CRLF to LF
    let normalized = content.replace("\r\n", "\n");

    for line in normalized.split('\n') {
        // Remove trailing whitespace from each line
        let trimmed = line.trim_end_matches([' ', '\t']);
        result.push_str(trimmed);
        result.push('\n');
    }

    // Remove the extra newline we added after the last line
    if result.ends_with('\n') && !normalized.ends_with('\n') {
        // Original didn't end with newline, but we want to add one
        // So keep the newline we added
    } else if result.ends_with("\n\n") && !normalized.ends_with("\n\n") {
        // We added an extra newline, remove it
        result.pop();
    }

    // Ensure exactly one final newline
    while result.ends_with("\n\n") {
        result.pop();
    }
    if !result.ends_with('\n') && !result.is_empty() {
        result.push('\n');
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_trailing_whitespace() {
        let input = "hello world   \nfoo bar\t\t\n";
        let expected = "hello world\nfoo bar\n";
        assert_eq!(fix_structural_issues(input), expected);
    }

    #[test]
    fn test_fix_crlf() {
        let input = "hello\r\nworld\r\n";
        let expected = "hello\nworld\n";
        assert_eq!(fix_structural_issues(input), expected);
    }

    #[test]
    fn test_add_final_newline() {
        let input = "hello world";
        let expected = "hello world\n";
        assert_eq!(fix_structural_issues(input), expected);
    }

    #[test]
    fn test_remove_extra_final_newlines() {
        let input = "hello world\n\n\n";
        let expected = "hello world\n";
        assert_eq!(fix_structural_issues(input), expected);
    }
}
