//! Census command - shows regime status

use crate::cli::CensusArgs;
use std::process::Command;

/// Native decrees that are always available
const NATIVE_DECREES: &[(&str, &[&str])] = &[
    ("supreme", &["*"]),
    ("ruby", &["rb", "rake"]),
    ("typescript", &["ts", "tsx", "js", "jsx"]),
    ("python", &["py"]),
    ("golang", &["go"]),
    ("rust", &["rs"]),
    ("frontmatter", &["md", "mdx"]),
];

/// External linters we can integrate with
const EXTERNAL_LINTERS: &[(&str, &str, &[&str])] = &[
    ("rubocop", "rubocop", &["rb"]),
    ("eslint", "eslint", &["ts", "tsx", "js", "jsx"]),
    ("ruff", "ruff", &["py"]),
    ("clippy", "cargo", &["rs"]),
    ("gofmt", "gofmt", &["go"]),
];

pub fn run_census(args: CensusArgs) {
    let config_path = args.config.unwrap_or_else(|| ".dictate.toml".into());
    let config_exists = config_path.exists();
    let dictate_config = dictator_core::DictateConfig::load_default();

    println!("Regime Status");
    println!("─────────────");
    println!();

    // Config status
    if config_exists {
        println!("Config: {config_path} (found)");
    } else {
        println!("Config: {config_path} (not found - using defaults)");
    }
    println!();

    // Native decrees
    println!("Native decrees: {}", NATIVE_DECREES.len());
    for (name, extensions) in NATIVE_DECREES {
        let enabled = dictate_config
            .as_ref()
            .is_none_or(|cfg| cfg.decree.contains_key(*name) || *name == "supreme");

        let status = if enabled { "✓" } else { "○" };
        let exts = extensions.join(", ");
        println!("  {status} {name:<12} (*.{exts})");
    }
    println!();

    // WASM decrees from config
    if let Some(ref cfg) = dictate_config {
        let wasm_decrees: Vec<_> = cfg
            .decree
            .iter()
            .filter(|(name, settings)| {
                settings.path.is_some() && !NATIVE_DECREES.iter().any(|(n, _)| *n == name.as_str())
            })
            .collect();

        if !wasm_decrees.is_empty() {
            println!("WASM decrees: {}", wasm_decrees.len());
            for (name, settings) in wasm_decrees {
                if let Some(ref path) = settings.path {
                    let exists = std::path::Path::new(path).exists();
                    let status = if exists { "✓" } else { "✗" };
                    println!("  {status} {name:<12} ({path})");
                }
            }
            println!();
        }
    }

    // External linters
    println!("External linters:");
    for (name, command, extensions) in EXTERNAL_LINTERS {
        let available = is_command_available(command);
        let configured = dictate_config.as_ref().is_some_and(|cfg| {
            // Check if any decree has this linter configured
            cfg.decree
                .values()
                .any(|d| d.linter.as_ref().is_some_and(|l| l.command == *command))
        });

        let status = match (available, configured) {
            (true, true) => "✓",
            (true, false) => "○",
            (false, _) => "✗",
        };

        let state = match (available, configured) {
            (true, true) => "configured",
            (true, false) => "available",
            (false, _) => "not found",
        };

        let exts = extensions.join(", ");
        println!("  {status} {name:<12} ({state}) [*.{exts}]");
    }
}

fn is_command_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
