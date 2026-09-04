#![warn(rust_2024_compatibility, clippy::all)]
#![allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::needless_pass_by_value
)]

mod census;
mod cli;
mod config;
mod dictate;
mod files;
mod interactive;
mod lint;
mod mcp;
mod mcp_host;
mod occupy;
mod output;
mod regime;
mod watch;

use anyhow::Result;

use census::run_census;
use cli::{Args, Command};
use dictate::run_dictate;
use lint::run_once;
use occupy::run_occupy;
use watch::run_watch;

fn main() -> Result<()> {
    use std::io::IsTerminal;

    // Detect MCP mode: stdin is a pipe (not terminal) and no CLI args
    // Works for both Claude Code and Codex
    let is_mcp = !std::io::stdin().is_terminal() && std::env::args().len() == 1;

    if is_mcp {
        // In MCP mode, log to stderr only (stdout is for JSON-RPC protocol)
        tracing_subscriber::fmt().with_writer(std::io::stderr).with_ansi(false).init();
        return mcp_host::run();
    }

    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let config = args.config;
    let profile = args.profile;
    match args.command {
        Command::Lint(lint) => run_once(lint, config, profile),
        Command::Dictate(dictate) => run_dictate(dictate, config, profile),
        Command::Watch(watch) => run_watch(watch, config, profile),
        Command::Census(census) => run_census(census, config, profile),
        Command::Occupy(occupy) => run_occupy(occupy),
        Command::Mcp => mcp_host::run(),
    }
}
