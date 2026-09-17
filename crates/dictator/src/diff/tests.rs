use super::*;

#[test]
fn parses_single_hunk() {
    let patch = "\
diff --git a/src/main.rs b/src/main.rs
index 1234567..89abcde 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,0 +11,3 @@ fn main() {
+    let a = 1;
+    let b = 2;
+    let c = 3;
";
    let parsed = parse_unified_diff(patch);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].0, Utf8PathBuf::from("src/main.rs"));
    assert_eq!(parsed[0].1, vec![11..=13]);
}

#[test]
fn parses_multiple_files_and_hunks() {
    let patch = "\
--- a/a.rb
+++ b/a.rb
@@ -1 +1 @@
-old
+new
@@ -20,2 +20,5 @@
+one
--- a/b.go
+++ b/b.go
@@ -5,0 +6,1 @@
+x
";
    let parsed = parse_unified_diff(patch);
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].0, Utf8PathBuf::from("a.rb"));
    assert_eq!(parsed[0].1, vec![1..=1, 20..=24]);
    assert_eq!(parsed[1].0, Utf8PathBuf::from("b.go"));
    assert_eq!(parsed[1].1, vec![6..=6]);
}

#[test]
fn hunk_without_count_is_one_line() {
    assert_eq!(hunk_new_range("@@ -3 +7 @@"), Some(7..=7));
}

#[test]
fn pure_deletion_hunk_is_skipped() {
    // `+12,0` means lines were removed and nothing added at line 12.
    assert_eq!(hunk_new_range("@@ -12,4 +12,0 @@"), None);
}

#[test]
fn deleted_file_produces_no_entry() {
    let patch = "\
--- a/gone.rs
+++ /dev/null
@@ -1,3 +0,0 @@
-a
-b
-c
";
    assert!(parse_unified_diff(patch).is_empty());
}

#[test]
fn rename_without_content_change_produces_no_entry() {
    let patch = "\
diff --git a/old.rs b/new.rs
similarity index 100%
rename from old.rs
rename to new.rs
";
    assert!(parse_unified_diff(patch).is_empty());
}

#[test]
fn hunk_header_with_section_heading_parses() {
    assert_eq!(
        hunk_new_range("@@ -100,7 +102,9 @@ impl Decree for Ruby {"),
        Some(102..=110)
    );
}

#[test]
fn path_with_spaces_keeps_its_spaces() {
    let parsed = parse_unified_diff("+++ b/dir/my file.rs\t\n@@ -0,0 +1,1 @@\n+x\n");
    assert_eq!(parsed[0].0, Utf8PathBuf::from("dir/my file.rs"));
}

#[test]
fn quoted_path_is_unescaped() {
    assert_eq!(unquote(r#""b/we\"ird.rs""#), r#"b/we"ird.rs"#);
    assert_eq!(unquote("b/plain.rs"), "b/plain.rs");
}

#[test]
fn context_expansion_merges_overlaps() {
    let merged = expand_and_merge(vec![10..=10, 13..=13, 40..=42], 2);
    // 8..=12 and 11..=15 overlap into 8..=15; 38..=44 stands alone.
    assert_eq!(merged, vec![8..=15, 38..=44]);
}

#[test]
fn context_expansion_clamps_at_line_one() {
    let expanded = expand_and_merge(vec![2..=2, 99..=99], 10);
    assert_eq!(expanded.first(), Some(&(1..=12)));
}

#[test]
fn adjacent_ranges_fuse() {
    assert_eq!(expand_and_merge(vec![5..=6, 7..=9], 0), vec![5..=9]);
}

#[test]
fn zero_context_leaves_ranges_alone() {
    assert_eq!(
        expand_and_merge(vec![5..=6, 30..=31], 0),
        vec![5..=6, 30..=31]
    );
}

#[test]
fn two_dot_range_takes_the_left_side() {
    assert_eq!(resolve_base("abc123..HEAD").unwrap(), "abc123");
}

#[test]
fn contains_line_respects_hunk_boundaries() {
    let path = Utf8PathBuf::from("/tmp/dictator-nonexistent/a.rs");
    let mut files = HashMap::new();
    files.insert(path.clone(), vec![10..=12, 20..=20]);
    let changed = ChangedLines { files };

    assert!(changed.contains_file(&path));
    assert!(changed.contains_line(&path, 10));
    assert!(changed.contains_line(&path, 12));
    assert!(changed.contains_line(&path, 20));
    assert!(!changed.contains_line(&path, 9));
    assert!(!changed.contains_line(&path, 13));
    assert!(!changed.contains_line(&path, 21));
    assert!(!changed.contains_file(Utf8Path::new("/tmp/dictator-nonexistent/b.rs")));
}
