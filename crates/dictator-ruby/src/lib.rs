//! Ruby hygiene rules implemented as a Dictator decree.

use dictator_decree_abi::{BoxDecree, Decree, Diagnostic, Diagnostics, Span};
use dictator_supreme::SupremeConfig;
use memchr::memchr_iter;

mod heredoc;

/// Configuration for ruby decree
#[derive(Debug, Clone)]
pub struct RubyConfig {
    pub max_lines: usize,
    pub ignore_comments: bool,
    /// Exempt heredoc bodies from `line-too-long` (RuboCop's `AllowHeredoc`).
    pub ignore_heredocs: bool,
    pub comment_spacing: bool,
}

impl Default for RubyConfig {
    fn default() -> Self {
        Self {
            max_lines: 300,
            ignore_comments: false,
            ignore_heredocs: false,
            comment_spacing: true,
        }
    }
}

/// Lint a Ruby source file and emit diagnostics for common hygiene issues.
#[must_use]
pub fn lint_source(source: &str) -> Diagnostics {
    lint_source_with_configs(source, &RubyConfig::default(), &SupremeConfig::default())
}

/// Lint with custom configuration
#[must_use]
pub fn lint_source_with_config(source: &str, config: &RubyConfig) -> Diagnostics {
    let mut diags = Diagnostics::new();

    diags.extend(dictator_supreme::lint_source_with_owner(
        source,
        &SupremeConfig::default(),
        "ruby",
    ));

    // Ruby-specific rules
    diags.extend(lint_ruby_specific(
        source,
        config,
        &heredoc::heredoc_lines(source),
    ));

    diags
}

#[must_use]
pub fn lint_source_with_configs(
    source: &str,
    ruby_config: &RubyConfig,
    supreme_config: &SupremeConfig,
) -> Diagnostics {
    let mut diags = Diagnostics::new();
    let heredoc = heredoc::heredoc_lines(source);

    let supreme_diags = dictator_supreme::lint_source_with_owner(source, supreme_config, "ruby");
    let lines: Vec<&str> = source.split('\n').collect();
    diags.extend(supreme_diags.into_iter().filter(|d| {
        if d.rule != "ruby/line-too-long" {
            return true;
        }
        let line_idx = source[..d.span.start].matches('\n').count();
        if heredoc[line_idx] {
            !ruby_config.ignore_heredocs
        } else {
            !(ruby_config.ignore_comments && lines[line_idx].trim_start().starts_with('#'))
        }
    }));

    // Ruby-specific rules
    diags.extend(lint_ruby_specific(source, ruby_config, &heredoc));

    diags
}

fn lint_ruby_specific(source: &str, config: &RubyConfig, heredoc: &[bool]) -> Diagnostics {
    let mut diags = Diagnostics::new();

    // Check file line count
    check_file_line_count(source, config.max_lines, heredoc, &mut diags);

    let bytes = source.as_bytes();
    let mut line_start: usize = 0;
    let mut line_idx: usize = 0;

    for nl in memchr_iter(b'\n', bytes) {
        process_line(
            source,
            line_start,
            nl,
            config.comment_spacing && !heredoc[line_idx],
            &mut diags,
        );
        line_start = nl + 1;
        line_idx += 1;
    }

    if line_start < bytes.len() {
        // Final line without trailing newline.
        process_line(
            source,
            line_start,
            bytes.len(),
            config.comment_spacing && !heredoc[line_idx],
            &mut diags,
        );
    }

    diags
}

/// Insert the missing space after `#` on every line `ruby/comment-space` flags.
#[must_use]
pub fn fix_comment_spacing(source: &str) -> String {
    let heredoc = heredoc::heredoc_lines(source);
    source
        .split('\n')
        .enumerate()
        .map(|(idx, line)| match unspaced_comment_hash(line) {
            Some(hash) if !heredoc[idx] => format!("{}# {}", &line[..hash], &line[hash + 1..]),
            _ => line.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Check file line count (excluding comments and blank lines)
fn check_file_line_count(
    source: &str,
    max_lines: usize,
    heredoc: &[bool],
    diags: &mut Diagnostics,
) {
    if max_lines == 0 {
        return; // max_lines = 0 disables the check
    }
    // A heredoc line starting with `#{` is string content, not a comment.
    let code_lines = source
        .split('\n')
        .zip(heredoc)
        .filter(|(line, in_heredoc)| {
            let trimmed = line.trim();
            !trimmed.is_empty() && (**in_heredoc || !trimmed.starts_with('#'))
        })
        .count();

    if code_lines > max_lines {
        diags.push(Diagnostic {
            rule: "ruby/file-too-long".to_string(),
            message: format!("{code_lines} code lines (max {max_lines})"),
            enforced: false,
            span: Span::new(0, source.len().min(100)),
        });
    }
}

#[derive(Default)]
pub struct RubyHygiene {
    config: RubyConfig,
    supreme: SupremeConfig,
}

impl RubyHygiene {
    #[must_use]
    pub const fn new(config: RubyConfig, supreme: SupremeConfig) -> Self {
        Self { config, supreme }
    }
}

impl Decree for RubyHygiene {
    fn name(&self) -> &'static str {
        "ruby"
    }

    fn lint(&self, _path: &str, source: &str) -> Diagnostics {
        lint_source_with_configs(source, &self.config, &self.supreme)
    }

    fn metadata(&self) -> dictator_decree_abi::DecreeMetadata {
        dictator_decree_abi::DecreeMetadata {
            abi_version: dictator_decree_abi::ABI_VERSION.to_string(),
            decree_version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Ruby code structure and hygiene".to_string(),
            persona: "Matz".to_string(),
            dectauthors: Some(env!("CARGO_PKG_AUTHORS").to_string()),
            supported_extensions: vec!["rb".to_string(), "rake".to_string(), "gemspec".to_string()],
            supported_filenames: vec![
                "Gemfile".to_string(),
                "Rakefile".to_string(),
                "Guardfile".to_string(),
                "Capfile".to_string(),
                "Brewfile".to_string(),
                "Dangerfile".to_string(),
                "Podfile".to_string(),
                "Fastfile".to_string(),
                "Appfile".to_string(),
                "Matchfile".to_string(),
                "Berksfile".to_string(),
                "Thorfile".to_string(),
                "Vagrantfile".to_string(),
                ".pryrc".to_string(),
                ".irbrc".to_string(),
            ],
            skip_filenames: vec!["Gemfile.lock".to_string(), "Podfile.lock".to_string()],
            file_scope_rules: dictator_supreme::file_scope_rules(&["file-too-long"]),
            capabilities: vec![dictator_decree_abi::Capability::Lint],
        }
    }
}

/// Factory used by host (native or WASM-exported).
#[must_use]
pub fn init_decree() -> BoxDecree {
    Box::new(RubyHygiene::default())
}

/// Create plugin with custom config
#[must_use]
pub fn init_decree_with_config(config: RubyConfig) -> BoxDecree {
    Box::new(RubyHygiene::new(config, SupremeConfig::default()))
}

/// Create plugin with custom config + supreme config (merged from decree.supreme + decree.ruby)
#[must_use]
pub fn init_decree_with_configs(config: RubyConfig, supreme: SupremeConfig) -> BoxDecree {
    Box::new(RubyHygiene::new(config, supreme))
}

/// Convert `DecreeSettings` to `RubyConfig`
#[must_use]
pub fn config_from_decree_settings(settings: &dictator_core::DecreeSettings) -> RubyConfig {
    RubyConfig {
        max_lines: settings.max_lines.unwrap_or(300),
        ignore_comments: settings.ignore_comments.unwrap_or(false),
        ignore_heredocs: settings.ignore_heredocs.unwrap_or(false),
        comment_spacing: settings.comment_spacing.unwrap_or(true),
    }
}

fn process_line(
    source: &str,
    start: usize,
    end: usize,
    comment_spacing: bool,
    diags: &mut Diagnostics,
) {
    if !comment_spacing {
        return;
    }

    // Comment hygiene: ensure space after '#', except for known directives.
    if let Some(hash) = unspaced_comment_hash(&source[start..end]) {
        // Span of the leading '#'
        let hash_offset = start + hash;
        diags.push(Diagnostic {
            rule: "ruby/comment-space".to_string(),
            message: "Comments should start with '# '".to_string(),
            enforced: true,
            span: Span::new(hash_offset, hash_offset + 1),
        });
    }
}

/// Byte offset of the `#` opening a comment that lacks the following space.
fn unspaced_comment_hash(line: &str) -> Option<usize> {
    let line = line.strip_suffix('\r').unwrap_or(line);
    let trimmed = line.trim_start_matches(' ');
    let rest = trimmed.strip_prefix('#')?;
    let unspaced = !rest.is_empty()
        && !rest.starts_with(' ')
        // `#{...}` opening a line of a multi-line string, not a comment
        && !rest.starts_with('{')
        && !is_comment_directive(rest);
    unspaced.then_some(line.len() - trimmed.len())
}

fn is_comment_directive(rest: &str) -> bool {
    let rest = rest.trim_start();

    rest.starts_with('!') // shebang
        || rest.starts_with("--") // RDoc: stop documenting
        || rest.starts_with("++") // RDoc: resume documenting
        || rest.starts_with("encoding")
        || rest.starts_with("frozen_string_literal")
        || rest.starts_with("rubocop")
        || rest.starts_with("typed")
}

#[cfg(test)]
mod tests;
