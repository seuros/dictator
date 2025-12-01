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
#[command(name = "dictator", version, about = "Multi-regime linter")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
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
}

#[derive(Debug, Parser)]
pub struct CensusArgs {
    /// Optional config file (TOML only). Default: .dictate.toml if present.
    #[arg(long)]
    pub config: Option<Utf8PathBuf>,
}

#[derive(Debug, Parser)]
pub struct LintArgs {
    /// Files or directories to lint.
    #[arg(required = true)]
    pub paths: Vec<Utf8PathBuf>,

    /// Optional config file (TOML only). Default: .dictate.toml if present.
    #[arg(long)]
    pub config: Option<Utf8PathBuf>,

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

    /// Optional config file (TOML only). Default: .dictate.toml if present.
    #[arg(long)]
    pub config: Option<Utf8PathBuf>,
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

    /// Optional config file (TOML only). Default: .dictate.toml if present.
    #[arg(long)]
    pub config: Option<Utf8PathBuf>,
}
