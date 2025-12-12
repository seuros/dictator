//! Census command - shows regime status

use crate::cli::CensusArgs;
use camino::Utf8PathBuf;
use dictator_core::DecreeSettings;
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

pub fn run_census(args: CensusArgs, config_path: Option<Utf8PathBuf>) -> anyhow::Result<()> {
    // Load decree configuration (with validation)
    let dictate_config = if let Some(p) = config_path.as_ref() {
        Some(dictator_core::DictateConfig::from_file(p.as_std_path())?)
    } else {
        dictator_core::DictateConfig::load_default_strict()?
    };

    let default_path = Utf8PathBuf::from(".dictate.toml");
    let config_display = config_path.as_ref().unwrap_or(&default_path);
    let config_exists = config_display.exists();

    println!("Regime Status");
    println!("─────────────");
    println!();

    // Config status
    if config_exists {
        println!("Config: {config_display} (found)");
    } else {
        println!("Config: {config_display} (not found - using defaults)");
    }
    println!();

    // Native decrees
    println!("Native decrees: {}", NATIVE_DECREES.len());
    for (name, extensions) in NATIVE_DECREES {
        // Mirror should_load_decree logic from regime.rs
        let (enabled, settings) = decree_status(dictate_config.as_ref(), name);

        let status = if enabled { "✓" } else { "○" };
        let exts = extensions.join(", ");
        println!("  {status} {name:<12} (*.{exts})");

        if args.details {
            print_settings(settings);
        }
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
                    if args.details {
                        print_settings(Some(settings));
                    }
                }
            }
            println!();
        }
    }

    // External linters (only those actually configured in .dictate.toml)
    println!("External linters:");
    let configured_linters = dictate_config
        .as_ref()
        .map(|cfg| {
            cfg.decree
                .iter()
                .filter_map(|(decree_name, settings)| {
                    settings
                        .linter
                        .as_ref()
                        .map(|l| (decree_name.as_str(), l.command.as_str()))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if configured_linters.is_empty() {
        println!("  (none configured)");
    } else {
        for (decree, command) in configured_linters {
            let available = is_command_available(command);
            let status = if available { "✓" } else { "✗" };
            let state = if available { "available" } else { "not found" };
            println!("  {status} {command:<12} ({state}, decree: {decree})");
        }
    }

    Ok(())
}

fn is_command_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn decree_status<'a>(
    cfg: Option<&'a dictator_core::DictateConfig>,
    name: &str,
) -> (bool, Option<&'a DecreeSettings>) {
    let settings = cfg.and_then(|c| c.decree.get(name));
    let enabled = name == "supreme" || settings.is_some_and(|s| s.enabled != Some(false));
    (enabled, settings)
}

fn print_settings(settings: Option<&DecreeSettings>) {
    let Some(settings) = settings else {
        println!("      (no overrides)");
        return;
    };

    let mut fields = Vec::new();

    macro_rules! push_opt {
        ($label:expr, $opt:expr) => {
            if let Some(val) = $opt {
                fields.push(format!("{}={}", $label, val));
            }
        };
    }

    push_opt!("enabled", settings.enabled);
    push_opt!("path", settings.path.as_deref());
    push_opt!(
        "trailing_whitespace",
        settings.trailing_whitespace.as_deref()
    );
    push_opt!("tabs_vs_spaces", settings.tabs_vs_spaces.as_deref());
    push_opt!("tab_width", settings.tab_width);
    push_opt!("final_newline", settings.final_newline.as_deref());
    push_opt!("line_endings", settings.line_endings.as_deref());
    push_opt!("max_line_length", settings.max_line_length);
    push_opt!(
        "blank_line_whitespace",
        settings.blank_line_whitespace.as_deref()
    );
    push_opt!("max_lines", settings.max_lines);
    push_opt!("ignore_comments", settings.ignore_comments);
    push_opt!("ignore_blank_lines", settings.ignore_blank_lines);

    if let Some(order) = settings.method_visibility_order.as_ref() {
        fields.push(format!("method_visibility_order=[{}]", order.join(",")));
    }
    if let Some(order) = settings.import_order.as_ref() {
        fields.push(format!("import_order=[{}]", order.join(",")));
    }
    if let Some(order) = settings.visibility_order.as_ref() {
        fields.push(format!("visibility_order=[{}]", order.join(",")));
    }
    if let Some(order) = settings.order.as_ref() {
        fields.push(format!("order=[{}]", order.join(",")));
    }
    if let Some(required) = settings.required.as_ref() {
        fields.push(format!("required=[{}]", required.join(",")));
    }
    if let Some(linter) = settings.linter.as_ref() {
        fields.push(format!("linter.command={}", linter.command));
    }

    if fields.is_empty() {
        println!("      (no overrides)");
    } else {
        for chunk in fields.chunks(4) {
            println!("      {}", chunk.join("  "));
        }
    }
}
