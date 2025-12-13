//! CLI argument parsing and command definitions

use camino::Utf8PathBuf;
use clap::Parser;
use serde::{Deserialize, Serialize};

/// Default debounce interval in milliseconds for watch mode.
/// Prevents multiple lints from running on rapid file changes.
pub const DEFAULT_DEBOUNCE_MS: u64 = 200;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputFormat {
    Human,
    Json,
}

#[derive(Debug, Parser)]
#[command(
    name = "dictator",
    version,
    about = "Multi-regime linter",
    disable_version_flag = true
)]
#[allow(clippy::manual_non_exhaustive)] // version field is for clap -v/--version
pub struct Args {
    /// Optional config file (TOML only). Default: .dictate.toml if present.
    #[arg(short, long, global = true)]
    pub config: Option<Utf8PathBuf>,

    #[command(subcommand)]
    pub command: Command,

    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: (),
}

#[derive(Debug, Parser)]
pub enum Command {
    /// Lint files/directories once and exit
    #[command(visible_alias = "stalint")]
    Lint(LintArgs),
    /// Fix structural issues (whitespace, newlines, line endings)
    #[command(visible_alias = "kjr")]
    Dictate(DictateArgs),
    /// Watch paths for changes and lint on the fly
    Watch(WatchArgs),
    /// Show regime status: loaded decrees, config, external linters
    Census(CensusArgs),
    /// Initialize .dictate.toml with default configuration
    #[command(visible_alias = "init")]
    Occupy(OccupyArgs),
    /// Run as MCP (Model Context Protocol) server
    Mcp,
}

#[derive(Debug, Parser)]
pub struct CensusArgs {
    /// Show decree configuration values from .dictate.toml
    #[arg(long)]
    pub details: bool,
}

#[derive(Debug, Parser)]
pub struct OccupyArgs {
    /// Target directory for .dictate.toml (defaults to current directory)
    #[arg(default_value = ".")]
    pub path: Utf8PathBuf,

    /// Overwrite existing .dictate.toml if present
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Debug, Parser)]
pub struct LintArgs {
    /// Files or directories to lint.
    #[arg(required = true)]
    pub paths: Vec<Utf8PathBuf>,

    /// Output JSON instead of human format
    #[arg(long)]
    pub json: bool,

    /// Load additional decrees (native .dylib/.so or .wasm when supported)
    #[cfg(feature = "wasm-loader")]
    #[arg(long, value_name = "PATH", num_args = 0..)]
    pub plugin: Vec<Utf8PathBuf>,
}

#[derive(Debug, Parser)]
pub struct DictateArgs {
    /// Files or directories to fix.
    #[arg(required = true)]
    pub paths: Vec<Utf8PathBuf>,
}

#[derive(Debug, Parser)]
pub struct WatchArgs {
    /// Paths to watch (files or directories). Defaults to current dir if omitted.
    #[arg(value_name = "PATH", default_value = ".")]
    pub paths: Vec<Utf8PathBuf>,

    /// Debounce interval in milliseconds
    #[arg(long, default_value_t = DEFAULT_DEBOUNCE_MS)]
    pub debounce_ms: u64,

    /// Output JSON instead of human format
    #[arg(long)]
    pub json: bool,

    /// Load additional decrees (native .dylib/.so or .wasm when supported)
    #[cfg(feature = "wasm-loader")]
    #[arg(long, value_name = "PATH", num_args = 0..)]
    pub plugin: Vec<Utf8PathBuf>,
}
