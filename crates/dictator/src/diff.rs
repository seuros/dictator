//! Git range resolution: which lines on the new side did this change touch?
//!
//! Always diffs base against the working tree, never `base..HEAD`, because decrees
//! parse the bytes on disk, so line numbers must describe those same bytes.

use crate::output::byte_to_line_col;
use anyhow::{Context, Result, bail};
use camino::{Utf8Path, Utf8PathBuf};
use dictator_decree_abi::Diagnostic;
use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;
use std::process::Command;

/// How the caller selected the range to lint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffSelector {
    /// A base revision, compared against the working tree.
    Base(String),
    /// Only what is staged for the next commit (`git diff --cached`).
    Staged,
}

/// Lines touched on the new side, keyed by canonicalized absolute path so keys
/// compare equal to whatever the file walker produced.
#[derive(Debug, Default, Clone)]
pub struct ChangedLines {
    files: HashMap<Utf8PathBuf, Vec<RangeInclusive<usize>>>,
}

impl ChangedLines {
    /// True when this path was added or modified by the range.
    #[must_use]
    pub fn contains_file(&self, path: &Utf8Path) -> bool {
        self.lookup(path).is_some()
    }

    /// True when this 1-indexed line sits inside a changed hunk.
    #[must_use]
    pub fn contains_line(&self, path: &Utf8Path, line: usize) -> bool {
        self.lookup(path)
            .is_some_and(|ranges| ranges.iter().any(|r| r.contains(&line)))
    }

    fn lookup(&self, path: &Utf8Path) -> Option<&Vec<RangeInclusive<usize>>> {
        if let Some(ranges) = self.files.get(path) {
            return Some(ranges);
        }
        // The walker may hand us a relative or non-canonical path; fall back to
        // canonicalizing before giving up.
        let canonical = canonicalize(path).ok()?;
        self.files.get(&canonical)
    }
}

/// Changed lines plus the decrees' file-scope rule names: everything needed to
/// decide whether a diagnostic belongs to the author of the diff.
pub struct Scope {
    changed: ChangedLines,
    file_scope: HashSet<String>,
}

impl Scope {
    #[must_use]
    pub fn new(changed: ChangedLines, file_scope: HashSet<String>) -> Self {
        Self {
            changed,
            file_scope,
        }
    }

    /// Does this rule describe the whole file rather than a line?
    #[must_use]
    pub fn is_file_scope(&self, rule: &str) -> bool {
        let bare = rule.rsplit('/').next().unwrap_or(rule);
        self.file_scope.contains(bare)
    }

    /// File-scope rules always report; the rest must overlap a changed hunk.
    #[must_use]
    pub fn allows(&self, path: &Utf8Path, text: &str, diag: &Diagnostic) -> bool {
        if self.is_file_scope(&diag.rule) {
            return true;
        }

        let (start, _) = byte_to_line_col(text, diag.span.start);
        let (end, _) = byte_to_line_col(text, diag.span.end.max(diag.span.start));
        (start..=end).any(|line| self.changed.contains_line(path, line))
    }
}

/// Build a selector from the `--diff`/`--staged` pair. `None` lints everything.
///
/// # Errors
///
/// Both flags supplied at once.
pub fn selector_from_flags(diff: Option<&str>, staged: bool) -> Result<Option<DiffSelector>> {
    match (diff, staged) {
        (Some(_), true) => bail!("--diff and --staged cannot be combined"),
        (Some(rev), false) => Ok(Some(DiffSelector::Base(rev.to_string()))),
        (None, true) => Ok(Some(DiffSelector::Staged)),
        (None, false) => Ok(None),
    }
}

/// Lines added or modified by `selector`, restricted to `paths`. `context`
/// grows every hunk by N lines on each side.
///
/// # Errors
///
/// Not a git repo, shallow clone, unresolvable base, or no `git` on PATH.
pub fn changed_lines(
    selector: &DiffSelector,
    paths: &[Utf8PathBuf],
    context: usize,
) -> Result<ChangedLines> {
    let root = repo_root()?;

    let mut args: Vec<String> = vec![
        // Keep non-ASCII paths verbatim instead of octal-escaping them.
        "-c".into(),
        "core.quotePath=false".into(),
        "diff".into(),
        "--unified=0".into(),
        "--no-color".into(),
        "--no-ext-diff".into(),
        "--find-renames".into(),
        // Added, Copied, Modified, Renamed. Deletions have no new side to lint.
        "--diff-filter=ACMR".into(),
    ];

    match selector {
        DiffSelector::Staged => args.push("--cached".into()),
        DiffSelector::Base(rev) => args.push(resolve_base(rev)?),
    }

    if !paths.is_empty() {
        args.push("--".into());
        args.extend(paths.iter().map(ToString::to_string));
    }

    let patch = git(&args)?;

    let mut files = HashMap::new();
    for (rel_path, ranges) in parse_unified_diff(&patch) {
        let absolute = root.join(&rel_path);
        // A file deleted between the diff and now has no content to lint.
        let Ok(key) = canonicalize(&absolute) else {
            continue;
        };
        files.insert(key, expand_and_merge(ranges, context));
    }

    // `git diff` omits untracked files, so a brand-new file in a dirty tree
    // would be skipped entirely. Every line of it is the author's work.
    if matches!(selector, DiffSelector::Base(_)) {
        for rel_path in untracked_files(paths)? {
            let Ok(key) = canonicalize(&root.join(&rel_path)) else {
                continue;
            };
            files.insert(key, vec![RangeInclusive::new(1, usize::MAX)]);
        }
    }

    Ok(ChangedLines { files })
}

/// Untracked, non-ignored files, repo-relative. Not staged, so `--staged`
/// correctly ignores these; a base-revision diff must not.
fn untracked_files(paths: &[Utf8PathBuf]) -> Result<Vec<Utf8PathBuf>> {
    let mut args = vec![
        "ls-files".to_string(),
        "--others".to_string(),
        "--exclude-standard".to_string(),
        "--full-name".to_string(),
    ];
    if !paths.is_empty() {
        args.push("--".into());
        args.extend(paths.iter().map(ToString::to_string));
    }
    Ok(git(&args)?.lines().map(Utf8PathBuf::from).collect())
}

/// Resolve `origin/master`, `HEAD~3`, `main...HEAD` or `main..HEAD` into a start
/// commit. Bare and three-dot forms go through `merge-base` so commits landing
/// on the base branch after the branch point aren't blamed on the author.
fn resolve_base(rev: &str) -> Result<String> {
    if let Some((left, right)) = rev.split_once("...") {
        let right = if right.is_empty() { "HEAD" } else { right };
        return merge_base(left, right);
    }
    if let Some((left, _)) = rev.split_once("..") {
        return Ok(left.to_string());
    }
    merge_base(rev, "HEAD")
}

fn merge_base(a: &str, b: &str) -> Result<String> {
    if is_shallow()? {
        bail!(
            "repository is a shallow clone, so the merge base with '{a}' is not present.\n\
             In GitHub Actions set `fetch-depth: 0` on actions/checkout; locally run \
             `git fetch --unshallow`."
        );
    }
    git(&["merge-base".to_string(), a.to_string(), b.to_string()])
        .map(|out| out.trim().to_string())
        .with_context(|| {
            format!(
                "could not find a merge base between '{a}' and '{b}'. \
                 Is '{a}' fetched? Try `git fetch origin {a}`."
            )
        })
}

fn is_shallow() -> Result<bool> {
    let out = git(&[
        "rev-parse".to_string(),
        "--is-shallow-repository".to_string(),
    ])?;
    Ok(out.trim() == "true")
}

fn repo_root() -> Result<Utf8PathBuf> {
    let out = git(&["rev-parse".to_string(), "--show-toplevel".to_string()])
        .context("not inside a git repository; --diff and --staged need one")?;
    canonicalize(Utf8Path::new(out.trim()))
}

fn canonicalize(path: &Utf8Path) -> Result<Utf8PathBuf> {
    let canonical =
        std::fs::canonicalize(path).with_context(|| format!("cannot canonicalize path: {path}"))?;
    Utf8PathBuf::from_path_buf(canonical)
        .map_err(|p| anyhow::anyhow!("non-utf8 path: {}", p.display()))
}

fn git(args: &[String]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context("failed to run `git`; is it installed and on PATH?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }

    String::from_utf8(output.stdout).context("git produced non-utf8 output")
}

/// Parse `git diff --unified=0` into per-file new-side ranges. Split from the
/// git call so it can be tested against fixture text.
fn parse_unified_diff(patch: &str) -> Vec<(Utf8PathBuf, Vec<RangeInclusive<usize>>)> {
    let mut out: Vec<(Utf8PathBuf, Vec<RangeInclusive<usize>>)> = Vec::new();
    let mut current: Option<usize> = None;

    for line in patch.lines() {
        if let Some(rest) = line.strip_prefix("+++ ") {
            current = match new_side_path(rest) {
                Some(path) => {
                    out.push((path, Vec::new()));
                    Some(out.len() - 1)
                }
                // `+++ /dev/null`: the file was deleted, nothing to lint.
                None => None,
            };
            continue;
        }

        if let Some(idx) = current
            && line.starts_with("@@")
            && let Some(range) = hunk_new_range(line)
        {
            out[idx].1.push(range);
        }
    }

    // A rename with no content change produces a header and no hunks.
    out.retain(|(_, ranges)| !ranges.is_empty());
    out
}

/// Extract the path from a `+++ b/path` header, or `None` for `/dev/null`.
fn new_side_path(rest: &str) -> Option<Utf8PathBuf> {
    // Trailing tabs appear when the path contains spaces.
    let rest = rest.trim_end_matches(['\r', '\t']);
    let unquoted = unquote(rest);
    if unquoted == "/dev/null" {
        return None;
    }
    // Strip the `b/` destination prefix that git prepends.
    let path = unquoted.strip_prefix("b/").unwrap_or(&unquoted);
    Some(Utf8PathBuf::from(path))
}

/// Undo git's C-style quoting of paths containing quotes or control characters.
fn unquote(raw: &str) -> String {
    let Some(inner) = raw.strip_prefix('"').and_then(|s| s.strip_suffix('"')) else {
        return raw.to_string();
    };

    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some(other) => out.push(other),
            None => break,
        }
    }
    out
}

/// New-side range from `@@ -a,b +c,d @@`. `None` for pure deletions (`+c,0`).
fn hunk_new_range(header: &str) -> Option<RangeInclusive<usize>> {
    let new_part = header
        .split_whitespace()
        .find(|part| part.starts_with('+'))?
        .trim_start_matches('+');

    let (start, count) = match new_part.split_once(',') {
        Some((start, count)) => (start.parse().ok()?, count.parse().ok()?),
        None => (new_part.parse().ok()?, 1usize),
    };

    if count == 0 {
        return None;
    }
    Some(start..=start + count - 1)
}

/// Grow each range by `context` lines, then coalesce the overlaps that creates.
fn expand_and_merge(
    ranges: Vec<RangeInclusive<usize>>,
    context: usize,
) -> Vec<RangeInclusive<usize>> {
    let mut expanded: Vec<RangeInclusive<usize>> = ranges
        .into_iter()
        .map(|r| {
            let start = r.start().saturating_sub(context).max(1);
            let end = r.end().saturating_add(context);
            start..=end
        })
        .collect();

    expanded.sort_by_key(|r| *r.start());

    let mut merged: Vec<RangeInclusive<usize>> = Vec::with_capacity(expanded.len());
    for range in expanded {
        match merged.last_mut() {
            // `+ 1` so adjacent ranges (5..=6, 7..=9) fuse into one.
            Some(last) if *range.start() <= last.end().saturating_add(1) => {
                if range.end() > last.end() {
                    *last = *last.start()..=*range.end();
                }
            }
            _ => merged.push(range),
        }
    }
    merged
}

#[cfg(test)]
mod tests;
