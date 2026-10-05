use super::*;

#[test]
fn test_interactive_fixer_creation() {
    let fixer = InteractiveFixer::new();
    assert_eq!(fixer.violations.len(), 0);
    assert_eq!(fixer.current_index, 0);
    assert!(!fixer.auto_apply_all);
}

fn violation(rule: &str, line: usize) -> FixableViolation {
    FixableViolation {
        path: Utf8PathBuf::from("x.rb"),
        line,
        column: 1,
        rule: rule.to_string(),
        message: String::new(),
        crlf: false,
    }
}

#[test]
fn map_line_keeps_terminators() {
    let upper = |l: &str| l.to_uppercase();
    assert_eq!(
        map_line("a\r\nb\nc", 1, upper).as_deref(),
        Some("A\r\nb\nc")
    );
    assert_eq!(
        map_line("a\r\nb\nc", 3, upper).as_deref(),
        Some("a\r\nb\nC")
    );
    assert_eq!(map_line("a\n", 2, upper), None);
    assert_eq!(map_line("a\n", 0, upper), None);
}

#[test]
fn fixes_on_one_line_compose() {
    let trailing = violation("ruby/trailing-whitespace", 1);
    let comment = violation("ruby/comment-space", 1);

    let (once, _) = trailing.fix("#one  \nx = 1\n").unwrap();
    assert_eq!(once, "#one\nx = 1\n");
    let (twice, _) = comment.fix(&once).unwrap();
    assert_eq!(twice, "# one\nx = 1\n");
    assert!(trailing.fix(&twice).is_none(), "already settled");
}

#[test]
fn final_newline_fix_is_not_repeated() {
    let newline = violation("ruby/missing-final-newline", 1);
    assert_eq!(newline.fix("x = 1").unwrap().0, "x = 1\n");
    assert!(newline.fix("x = 1\n").is_none());
}

#[test]
fn applying_every_fix_keeps_every_fix() {
    let dir = tempfile::tempdir().unwrap();
    let path = Utf8PathBuf::from_path_buf(dir.path().join("i.rb")).unwrap();
    fs::write(&path, "#one  \nx = 1\n#two\ny = 2   \n").unwrap();

    // Language decrees only load when configured.
    let mut config = DictateConfig::default();
    config.decree.insert("ruby".to_string(), Default::default());

    let mut fixer = InteractiveFixer::new();
    fixer
        .collect_violations(std::slice::from_ref(&path), None, Some(&config))
        .unwrap();
    assert_eq!(fixer.violations.len(), 4);
    fixer.run_interactive(true).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "# one\nx = 1\n# two\ny = 2\n"
    );
}

#[test]
fn line_ending_fix_follows_the_owner_config() {
    let mut to_crlf = violation("supreme/wrong-line-ending", 1);
    to_crlf.crlf = true;
    assert_eq!(to_crlf.fix("a\nb\n").unwrap().0, "a\r\nb\r\n");

    let to_lf = violation("supreme/mixed-line-endings", 1);
    assert_eq!(to_lf.fix("a\r\nb\n").unwrap().0, "a\nb\n");
}

#[test]
fn final_newline_follows_crlf_files() {
    let newline = violation("supreme/missing-final-newline", 1);
    assert_eq!(newline.fix("a\r\nb").unwrap().0, "a\r\nb\r\n");
}
