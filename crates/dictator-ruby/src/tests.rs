use super::*;

#[test]
fn detects_trailing_whitespace_and_tab() {
    let src = "def foo\n  bar \t\nend\n";
    let diags = lint_source(src);
    assert!(diags.iter().any(|d| d.rule == "ruby/trailing-whitespace"));
    assert!(diags.iter().any(|d| d.rule == "ruby/tab-character"));
}

#[test]
fn detects_missing_final_newline() {
    let src = "class Foo\nend";
    let diags = lint_source(src);
    assert!(diags.iter().any(|d| d.rule == "ruby/missing-final-newline"));
}

#[test]
fn enforces_comment_space() {
    let src = "#bad\n# good\n";
    let diags = lint_source(src);
    assert!(diags.iter().any(|d| d.rule == "ruby/comment-space"));
}

#[test]
fn comment_spacing_false_disables_comment_space_check() {
    let src = "#bad\n## Section\n";
    let config = RubyConfig {
        comment_spacing: false,
        ..Default::default()
    };
    let supreme = SupremeConfig::default();
    let diags = lint_source_with_configs(src, &config, &supreme);
    assert!(!diags.iter().any(|d| d.rule == "ruby/comment-space"));
}

#[test]
fn config_from_decree_settings_honors_comment_spacing() {
    let settings = dictator_core::DecreeSettings {
        comment_spacing: Some(false),
        ..Default::default()
    };
    let config = config_from_decree_settings(&settings);
    assert!(!config.comment_spacing);
}

#[test]
fn ignores_long_comment_lines_when_configured() {
    let long_comment = format!("# {}\n", "x".repeat(150));
    let src = format!("def foo\n{long_comment}end\n");
    let config = RubyConfig {
        ignore_comments: true,
        ..Default::default()
    };
    let supreme = SupremeConfig {
        max_line_length: Some(120),
        ..Default::default()
    };
    let diags = lint_source_with_configs(&src, &config, &supreme);
    assert!(!diags.iter().any(|d| d.rule == "ruby/line-too-long"));
}

#[test]
fn detects_long_comment_lines_when_not_configured() {
    let long_comment = format!("# {}\n", "x".repeat(150));
    let src = format!("def foo\n{long_comment}end\n");
    let config = RubyConfig::default(); // ignore_comments = false
    let supreme = SupremeConfig {
        max_line_length: Some(120),
        ..Default::default()
    };
    let diags = lint_source_with_configs(&src, &config, &supreme);
    assert!(diags.iter().any(|d| d.rule == "ruby/line-too-long"));
}

#[test]
fn still_detects_long_code_lines_with_ignore_comments() {
    let long_code = format!("  x = \"{}\"\n", "a".repeat(150));
    let src = format!("def foo\n{long_code}end\n");
    let config = RubyConfig {
        ignore_comments: true,
        ..Default::default()
    };
    let supreme = SupremeConfig {
        max_line_length: Some(120),
        ..Default::default()
    };
    let diags = lint_source_with_configs(&src, &config, &supreme);
    assert!(diags.iter().any(|d| d.rule == "ruby/line-too-long"));
}

#[test]
fn detects_whitespace_only_blank_line() {
    let src = "def foo\n  bar\n    \nend\n"; // blank line has spaces
    let diags = lint_source(src);
    assert!(diags.iter().any(|d| d.rule == "ruby/blank-line-whitespace"));
}

fn long_sql_heredoc() -> String {
    format!(
        "def sql\n  <<~SQL\n    #{{table}} (\n    #{{cols}} {}\n  SQL\nend\n",
        "x".repeat(150)
    )
}

fn with_max_line_length(src: &str, config: &RubyConfig) -> Diagnostics {
    let supreme = SupremeConfig {
        max_line_length: Some(120),
        ..Default::default()
    };
    lint_source_with_configs(src, config, &supreme)
}

#[test]
fn heredoc_interpolation_is_not_a_comment() {
    let diags = lint_source(&long_sql_heredoc());
    assert!(!diags.iter().any(|d| d.rule == "ruby/comment-space"));
}

#[test]
fn flags_long_heredoc_lines_by_default() {
    let config = RubyConfig {
        ignore_comments: true,
        ..Default::default()
    };
    let diags = with_max_line_length(&long_sql_heredoc(), &config);
    assert!(diags.iter().any(|d| d.rule == "ruby/line-too-long"));
}

#[test]
fn ignore_heredocs_exempts_long_heredoc_lines() {
    let config = RubyConfig {
        ignore_heredocs: true,
        ..Default::default()
    };
    let src = format!("{}  x = \"{}\"\n", long_sql_heredoc(), "a".repeat(150));
    let diags = with_max_line_length(&src, &config);
    let long: Vec<_> = diags
        .iter()
        .filter(|d| d.rule == "ruby/line-too-long")
        .collect();
    assert_eq!(long.len(), 1, "only the code line outside the heredoc");
}

#[test]
fn heredoc_interpolation_lines_count_as_code() {
    let body = "    #{col},\n".repeat(60);
    let src = format!("x = <<~SQL\n{body}SQL\n");
    let config = RubyConfig {
        max_lines: 50,
        ..Default::default()
    };
    let diags = lint_source_with_configs(&src, &config, &SupremeConfig::default());
    assert!(diags.iter().any(|d| d.rule == "ruby/file-too-long"));
}

#[test]
fn config_from_decree_settings_honors_ignore_heredocs() {
    let settings = dictator_core::DecreeSettings {
        ignore_heredocs: Some(true),
        ..Default::default()
    };
    assert!(config_from_decree_settings(&settings).ignore_heredocs);
    assert!(
        !config_from_decree_settings(&dictator_core::DecreeSettings::default()).ignore_heredocs
    );
}

#[test]
fn max_lines_zero_disables_file_too_long() {
    let src = "x = 1\n".repeat(1000);
    let config = RubyConfig {
        max_lines: 0,
        ..Default::default()
    };
    let diags = lint_source_with_config(&src, &config);
    assert!(!diags.iter().any(|d| d.rule == "ruby/file-too-long"));
}

#[test]
fn fix_comment_spacing_inserts_the_missing_space() {
    let src = "#bad\n  #also bad\n# good\n#\nx = 1 #trailing\n";
    assert_eq!(
        fix_comment_spacing(src),
        "# bad\n  # also bad\n# good\n#\nx = 1 #trailing\n"
    );
}

#[test]
fn fix_comment_spacing_leaves_non_comments_alone() {
    let src = "#!/usr/bin/env ruby\n#frozen_string_literal: true\n#--\n#++\n\
               sql = <<~SQL\n#raw\nSQL\ns = \"a\n#{b}\"\r\n";
    assert_eq!(fix_comment_spacing(src), src);
    assert!(
        !lint_source(src)
            .iter()
            .any(|d| d.rule == "ruby/comment-space")
    );
}

#[test]
fn fix_comment_spacing_preserves_crlf_and_clears_the_lint() {
    let src = "#bad\r\nx = 1\r\n";
    let fixed = fix_comment_spacing(src);
    assert_eq!(fixed, "# bad\r\nx = 1\r\n");
    assert!(
        !lint_source(&fixed)
            .iter()
            .any(|d| d.rule == "ruby/comment-space")
    );
}
