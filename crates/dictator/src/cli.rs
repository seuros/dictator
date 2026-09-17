//! CLI argument parsing and command definitions

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

/// Default debounce interval in milliseconds for watch mode.
/// Prevents multiple lints from running on rapid file changes.
pub const DEFAULT_DEBOUNCE_MS: u64 = 200;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    Json,
    /// GitHub Actions workflow commands, rendered as PR annotations.
    Github,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            "github" => Ok(Self::Github),
            other => Err(format!("unknown format '{other}' (human, json, github)")),
        }
    }
}

/// Multi-regime linter
#[derive(Debug, usage::Cli)]
#[usage(
    bin = "dictator",
    version,
    unknown_flags = "error",
    args_override_self = false
)]
pub struct Args {
    /// Optional config file (TOML only). Default: .dictate.toml if present.
    #[usage(short, long, global)]
    pub config: Option<Utf8PathBuf>,

    /// Configuration profile to use (e.g., strict, relaxed, ci)
    #[usage(short = 'p', long, global)]
    pub profile: Option<String>,

    #[usage(subcommand)]
    pub command: Command,
}

#[derive(Debug, usage::Subcommands)]
pub enum Command {
    /// Lint files/directories once and exit
    #[usage(visible_alias = "stalint")]
    Lint(LintArgs),
    /// Fix structural issues (whitespace, newlines, line endings)
    #[usage(visible_alias = "kjr")]
    Dictate(DictateArgs),
    /// Watch paths for changes and lint on the fly
    Watch(WatchArgs),
    /// Show regime status: loaded decrees, config, external linters
    Census(CensusArgs),
    /// Initialize .dictate.toml with default configuration
    #[usage(visible_alias = "init")]
    Occupy(OccupyArgs),
    /// Run as MCP (Model Context Protocol) server
    Mcp,
}

#[derive(Debug, usage::Args)]
pub struct CensusArgs {
    /// Show decree configuration values from .dictate.toml
    #[usage(long)]
    pub details: bool,
}

#[derive(Debug, usage::Args)]
pub struct OccupyArgs {
    /// Target directory for .dictate.toml (defaults to current directory)
    #[usage(default = ".")]
    pub path: Utf8PathBuf,

    /// Overwrite existing .dictate.toml if present
    #[usage(short, long)]
    pub force: bool,
}

#[derive(Debug, usage::Args)]
pub struct LintArgs {
    /// Files or directories to lint. Defaults to "." with --diff/--staged.
    pub paths: Vec<Utf8PathBuf>,

    /// Auto-fix structural violations after linting
    #[usage(short = 'f', long)]
    pub fix: bool,

    /// Output format: human, json, or github (PR annotations)
    #[usage(long, value_name = "FMT")]
    pub format: Option<String>,

    /// Only report lines changed since REV (e.g. origin/master, HEAD~3)
    #[usage(long, value_name = "REV")]
    pub diff: Option<String>,

    /// Only report lines staged for commit
    #[usage(long)]
    pub staged: bool,

    /// Widen each changed hunk by N lines
    #[usage(long, default_value_t = 0, default = "0")]
    pub diff_context: usize,

    /// Load additional decrees (native .dylib/.so or .wasm when supported)
    #[cfg(feature = "wasm-loader")]
    #[usage(long, value_name = "PATH", variadic)]
    pub plugin: Vec<Utf8PathBuf>,
}

#[derive(Debug, usage::Args)]
pub struct DictateArgs {
    /// Files or directories to fix. Defaults to "." with --diff/--staged.
    pub paths: Vec<Utf8PathBuf>,

    /// Interactive mode - review each fix before applying
    #[usage(short, long)]
    pub interactive: bool,

    /// Only fix lines changed since REV (e.g. origin/master, HEAD~3)
    #[usage(long, value_name = "REV")]
    pub diff: Option<String>,

    /// Only fix lines staged for commit
    #[usage(long)]
    pub staged: bool,

    /// Widen each changed hunk by N lines
    #[usage(long, default_value_t = 0, default = "0")]
    pub diff_context: usize,
}

#[derive(Debug, usage::Args)]
pub struct WatchArgs {
    /// Paths to watch (files or directories). Defaults to current dir if omitted.
    #[usage(value_name = "PATH", default = ".")]
    pub paths: Vec<Utf8PathBuf>,

    /// Debounce interval in milliseconds
    #[usage(long, default_value_t = DEFAULT_DEBOUNCE_MS, default = "200")]
    pub debounce_ms: u64,

    /// Output format: human, json, or github (PR annotations)
    #[usage(long, value_name = "FMT")]
    pub format: Option<String>,

    /// Load additional decrees (native .dylib/.so or .wasm when supported)
    #[cfg(feature = "wasm-loader")]
    #[usage(long, value_name = "PATH", variadic)]
    pub plugin: Vec<Utf8PathBuf>,
}
