//! Watch command implementation - file watching and live linting

use anyhow::Result;
use camino::Utf8PathBuf;
use dictator_core::Source;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use notify_types::event::{Event, EventKind};
use std::collections::HashSet;
use std::fs;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::cli::{OutputFormat, WatchArgs};
use crate::config::load_config;
use crate::output::{SerializableDiagnostic, byte_to_line_col, print_diagnostic};
use crate::regime::init_regime_for_watch;

/// Timeout duration for receiving events from the file watcher in watch mode.
/// Controls how often we check if there are pending file changes.
const WATCHER_RX_TIMEOUT_MS: u64 = 200;

pub fn run_watch(args: WatchArgs, config_path: Option<Utf8PathBuf>) -> Result<()> {
    let cfg = load_config(config_path.as_ref())?;
    let format = if args.json {
        OutputFormat::Json
    } else {
        cfg.format.unwrap_or(OutputFormat::Human)
    };

    // Load decree configuration
    let decree_config = config_path
        .as_ref()
        .and_then(|p| dictator_core::DictateConfig::from_file(p.as_std_path()).ok())
        .or_else(dictator_core::DictateConfig::load_default);

    // Load native decrees
    let mut regime = init_regime_for_watch(decree_config.as_ref());

    // Extensions actually covered by loaded decrees (empty/None = watch all)
    let watched_exts = regime.watched_extensions();

    // Load custom WASM decrees from config
    if let Some(ref config) = decree_config {
        for (name, settings) in &config.decree {
            // Skip built-in native decrees
            if matches!(
                name.as_str(),
                "supreme" | "ruby" | "typescript" | "golang" | "rust" | "python" | "frontmatter"
            ) {
                continue;
            }

            // Load custom decree if path provided and enabled
            if let Some(ref path) = settings.path
                && settings.enabled.unwrap_or(true)
            {
                regime.add_wasm_decree(path)?;
            }
        }
    }

    // Load any additional decrees from CLI
    for p in &args.plugin {
        regime.add_wasm_decree(p)?;
    }

    let (tx, rx) = mpsc::channel();
    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                tx.send(event).ok();
            }
        })?;

    for path in &args.paths {
        watcher.watch(path.as_std_path(), RecursiveMode::Recursive)?;
    }

    println!("Watching {} path(s) for code changes...", args.paths.len());

    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    {
        let running = running.clone();
        ctrlc::set_handler(move || {
            running.store(false, std::sync::atomic::Ordering::SeqCst);
        })?;
    }

    let mut last_run = Instant::now();
    while running.load(std::sync::atomic::Ordering::SeqCst) {
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(WATCHER_RX_TIMEOUT_MS)) {
            if !is_relevant(&event) {
                continue;
            }

            // Debounce bursts (wait half the debounce interval since last run)
            if last_run.elapsed() < Duration::from_millis(args.debounce_ms / 2) {
                continue;
            }

            let mut files = Vec::new();
            for path in &event.paths {
                let Ok(p) = Utf8PathBuf::from_path_buf(path.to_owned()) else {
                    continue;
                };

                let ext = p.extension().map(str::to_ascii_lowercase);

                // If specific extensions are declared, only watch those; otherwise watch all.
                if let Some(ref allowed) = watched_exts {
                    let Some(ext) = ext else { continue; };
                    if !allowed.contains(ext.as_str()) {
                        continue;
                    }
                }

                files.push(p);
            }
            if files.is_empty() {
                continue;
            }

            let mut exit_code = 0;
            let mut json_out: Vec<SerializableDiagnostic> = Vec::new();
            for path in files {
                // If the file vanished between the event and processing, ignore it.
                if !std::path::Path::new(path.as_str()).exists() {
                    continue;
                }

                let text = fs::read_to_string(&path)?;
                let path_ref = path.as_path();
                let source = Source {
                    path: path_ref,
                    text: &text,
                };
                let diags = regime.enforce(&[source])?;
                let mut seen = HashSet::new();
                for diag in diags {
                    let key = (
                        path_ref.as_str().to_string(),
                        diag.span.start,
                        diag.span.end,
                        diag.rule.clone(),
                        diag.message.clone(),
                        diag.enforced,
                    );
                    if !seen.insert(key) {
                        continue;
                    }
                    match format {
                        OutputFormat::Human => {
                            print_diagnostic(path_ref.as_str(), &text, &diag);
                        }
                        OutputFormat::Json => {
                            let (line, col) = byte_to_line_col(&text, diag.span.start);
                            json_out.push(SerializableDiagnostic {
                                path: path_ref.as_str().to_string(),
                                line,
                                col,
                                rule: diag.rule.clone(),
                                message: diag.message.clone(),
                                enforced: diag.enforced,
                                span: diag.span,
                            });
                        }
                    }
                    exit_code = 1;
                }
            }

            if matches!(format, OutputFormat::Json) {
                println!("{}", serde_json::to_string_pretty(&json_out)?);
            }

            if exit_code != 0 {
                println!("lint failures (exit {exit_code})");
            }

            last_run = Instant::now();
        }
    }

    Ok(())
}

const fn is_relevant(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Modify(_) | EventKind::Create(_)
    )
}
