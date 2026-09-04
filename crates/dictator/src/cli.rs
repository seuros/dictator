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
}

/// Multi-regime linter
#[derive(Debug, usage::Cli)]
#[usage(bin = "dictator", version, unknown_flags = "error", args_override_self = false)]
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
    /// Files or directories to lint.
    #[usage(required)]
    pub paths: Vec<Utf8PathBuf>,

    /// Auto-fix structural violations after linting
    #[usage(short = 'f', long)]
    pub fix: bool,

    /// Output JSON instead of human format
    #[usage(long)]
    pub json: bool,

    /// Load additional decrees (native .dylib/.so or .wasm when supported)
    #[cfg(feature = "wasm-loader")]
    #[usage(long, value_name = "PATH", variadic)]
    pub plugin: Vec<Utf8PathBuf>,
}

#[derive(Debug, usage::Args)]
pub struct DictateArgs {
    /// Files or directories to fix.
    #[usage(required)]
    pub paths: Vec<Utf8PathBuf>,

    /// Interactive mode - review each fix before applying
    #[usage(short, long)]
    pub interactive: bool,
}

#[derive(Debug, usage::Args)]
pub struct WatchArgs {
    /// Paths to watch (files or directories). Defaults to current dir if omitted.
    #[usage(value_name = "PATH", default = ".")]
    pub paths: Vec<Utf8PathBuf>,

    /// Debounce interval in milliseconds
    #[usage(long, default_value_t = DEFAULT_DEBOUNCE_MS, default = "200")]
    pub debounce_ms: u64,

    /// Output JSON instead of human format
    #[usage(long)]
    pub json: bool,

    /// Load additional decrees (native .dylib/.so or .wasm when supported)
    #[cfg(feature = "wasm-loader")]
    #[usage(long, value_name = "PATH", variadic)]
    pub plugin: Vec<Utf8PathBuf>,
}
