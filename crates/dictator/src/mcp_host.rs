//! MCP host implementation - dictator as guest using mcp-host framework

mod prompts;
mod resources;
mod server;
mod tools;

pub use server::run;

/// Check if .dictate.toml exists in current directory
fn config_exists() -> bool {
    std::env::current_dir()
        .map(|cwd| cwd.join(".dictate.toml").exists())
        .unwrap_or(false)
}
