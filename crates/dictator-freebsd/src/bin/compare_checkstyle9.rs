use std::cmp::min;
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use dictator_decree_abi::Decree;
use dictator_freebsd::FreeBsdDecree;
use dictator_supreme::{LineEnding, SupremeConfig, TabsOrSpaces, lint_source_with_config};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Severity {
    Error,
    Warning,
}

impl Severity {
    fn as_str(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warning => "WARNING",
        }
    }
}

#[derive(Clone, Debug)]
struct FlatDiag {
    file: String,
    line: usize,
    severity: Severity,
    message: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Key {
    line: usize,
    severity: Severity,
    message: String,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct LmKey {
    line: usize,
    message: String,
}

#[derive(Default)]
struct Totals {
    files_scanned: usize,
    files_with_diags: usize,
    perl_diags: usize,
    rust_diags: usize,
    exact_matches: usize,
    severity_mismatches: usize,
    missing_in_rust: usize,
    extra_in_rust: usize,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut src_root = PathBuf::from("/usr/src");
    let mut limit = Some(300usize);
    let mut batch_size = 100usize;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--src" => {
                let Some(v) = args.next() else {
                    return Err("--src requires a value".to_string());
                };
                src_root = PathBuf::from(v);
            }
            "--limit" => {
                let Some(v) = args.next() else {
                    return Err("--limit requires a value".to_string());
                };
                limit = Some(
                    v.parse::<usize>()
                        .map_err(|_| format!("invalid --limit: {v}"))?,
                );
            }
            "--all" => {
                limit = None;
            }
            "--batch" => {
                let Some(v) = args.next() else {
                    return Err("--batch requires a value".to_string());
                };
                batch_size = v
                    .parse::<usize>()
                    .map_err(|_| format!("invalid --batch: {v}"))?;
                if batch_size == 0 {
                    return Err("--batch must be > 0".to_string());
                }
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            other => {
                return Err(format!("unknown argument: {other}"));
            }
        }
    }

    if !src_root.exists() {
        return Err(format!(
            "source root does not exist: {}",
            src_root.display()
        ));
    }

    let t0 = Instant::now();
    let mut files = Vec::new();
    collect_c_and_h_files(&src_root, &mut files);
    files.sort();

    if let Some(n) = limit {
        files.truncate(n);
    }

    if files.is_empty() {
        return Err(format!("no .c/.h files found under {}", src_root.display()));
    }

    let discover_elapsed = t0.elapsed();
    eprintln!(
        "discovered {} files under {} in {:?}",
        files.len(),
        src_root.display(),
        discover_elapsed
    );

    let t1 = Instant::now();
    let perl_diags = run_checkstyle_batches(&files, batch_size)?;
    let perl_elapsed = t1.elapsed();

    let t2 = Instant::now();
    let rust_diags = run_rust_linter(&files)?;
    let rust_elapsed = t2.elapsed();

    let t3 = Instant::now();
    let report = compare(perl_diags, rust_diags, files.len());
    let cmp_elapsed = t3.elapsed();

    print_report(&report);
    println!();
    println!("timings:");
    println!("- discover: {:?}", discover_elapsed);
    println!("- checkstyle9: {:?}", perl_elapsed);
    println!("- rust-linter: {:?}", rust_elapsed);
    println!("- compare: {:?}", cmp_elapsed);

    Ok(())
}

fn print_help() {
    println!("compare_checkstyle9");
    println!();
    println!("Compare this decree output with FreeBSD tools/build/checkstyle9.pl");
    println!();
    println!("Options:");
    println!("  --src <path>    Source tree root (default: /usr/src)");
    println!("  --limit <n>     Number of .c/.h files to check (default: 300)");
    println!("  --all           Check all .c/.h files");
    println!("  --batch <n>     Files per checkstyle9 invocation (default: 100)");
}

fn collect_c_and_h_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(v) => v,
        Err(_) => return,
    };

    for ent in entries.flatten() {
        let path = ent.path();
        let ftype = match ent.file_type() {
            Ok(v) => v,
            Err(_) => continue,
        };

        if ftype.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            collect_c_and_h_files(&path, out);
            continue;
        }

        if !ftype.is_file() {
            continue;
        }

        if matches!(path.extension().and_then(OsStr::to_str), Some("c" | "h")) {
            out.push(path);
        }
    }
}

fn should_skip_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(OsStr::to_str),
        Some(".git" | ".svn" | ".hg")
    )
}

fn run_checkstyle_batches(files: &[PathBuf], batch_size: usize) -> Result<Vec<FlatDiag>, String> {
    let mut diags = Vec::new();

    for chunk in files.chunks(batch_size) {
        let mut cmd = Command::new("perl");
        cmd.arg("/usr/src/tools/build/checkstyle9.pl")
            .arg("--file")
            .arg("--terse")
            .arg("--no-summary")
            .arg("--color=never");

        for file in chunk {
            cmd.arg(file);
        }

        let out = cmd
            .output()
            .map_err(|e| format!("failed to run checkstyle9.pl: {e}"))?;

        let mut text = String::new();
        text.push_str(&String::from_utf8_lossy(&out.stdout));
        if !out.stderr.is_empty() {
            text.push('\n');
            text.push_str(&String::from_utf8_lossy(&out.stderr));
        }

        for line in text.lines() {
            if let Some(diag) = parse_checkstyle_line(line) {
                diags.push(diag);
            }
        }
    }

    Ok(diags)
}

fn parse_checkstyle_line(line: &str) -> Option<FlatDiag> {
    let clean = strip_ansi(line).trim().to_string();

    let (prefix, sev, msg) = if let Some((p, m)) = clean.split_once(": ERROR: ") {
        (p, Severity::Error, m.to_string())
    } else if let Some((p, m)) = clean.split_once(": WARNING: ") {
        (p, Severity::Warning, m.to_string())
    } else {
        return None;
    };

    let (file, line_s) = prefix.rsplit_once(':')?;
    let line_no = line_s.parse::<usize>().ok()?;

    Some(FlatDiag {
        file: file.to_string(),
        line: line_no,
        severity: sev,
        message: msg,
    })
}

fn strip_ansi(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == 0x1b {
            i += 1;
            if i < bytes.len() && bytes[i] == b'[' {
                i += 1;
                while i < bytes.len() {
                    let b = bytes[i];
                    i += 1;
                    if b.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }

        out.push(bytes[i] as char);
        i += 1;
    }

    out
}

fn run_rust_linter(files: &[PathBuf]) -> Result<Vec<FlatDiag>, String> {
    let decree = FreeBsdDecree;
    let supreme_cfg = SupremeConfig {
        max_line_length: None,
        trailing_whitespace: true,
        tabs_vs_spaces: TabsOrSpaces::Either,
        final_newline: false,
        blank_line_whitespace: false,
        line_endings: LineEnding::Either,
    };
    let mut out = Vec::new();

    for file in files {
        let src = fs::read_to_string(file)
            .map_err(|e| format!("failed to read {}: {e}", file.display()))?;
        let line_starts = compute_line_starts(&src);
        let path_str = file.to_string_lossy().to_string();

        for d in decree.lint(&path_str, &src) {
            let line = line_for_offset(&line_starts, d.span.start);
            let sev = if d.enforced {
                Severity::Error
            } else {
                Severity::Warning
            };
            out.push(FlatDiag {
                file: path_str.clone(),
                line,
                severity: sev,
                message: d.message,
            });
        }

        for d in lint_source_with_config(&src, &supreme_cfg) {
            let line = line_for_offset(&line_starts, d.span.start);
            let sev = if d.enforced {
                Severity::Error
            } else {
                Severity::Warning
            };
            out.push(FlatDiag {
                file: path_str.clone(),
                line,
                severity: sev,
                message: d.message,
            });
        }
    }

    Ok(out)
}

fn compute_line_starts(src: &str) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, b) in src.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

fn line_for_offset(starts: &[usize], offset: usize) -> usize {
    match starts.binary_search(&offset) {
        Ok(i) => i + 1,
        Err(i) => i,
    }
}

struct CompareReport {
    totals: Totals,
    top_missing: Vec<(String, usize)>,
    top_extra: Vec<(String, usize)>,
    mismatch_examples: Vec<String>,
    missing_examples: Vec<String>,
    extra_examples: Vec<String>,
}

fn compare(
    perl_diags: Vec<FlatDiag>,
    rust_diags: Vec<FlatDiag>,
    files_scanned: usize,
) -> CompareReport {
    let mut perl_by_file: HashMap<String, Vec<FlatDiag>> = HashMap::new();
    let mut rust_by_file: HashMap<String, Vec<FlatDiag>> = HashMap::new();

    for d in perl_diags {
        perl_by_file.entry(d.file.clone()).or_default().push(d);
    }
    for d in rust_diags {
        rust_by_file.entry(d.file.clone()).or_default().push(d);
    }

    let mut all_files: BTreeMap<String, ()> = BTreeMap::new();
    for f in perl_by_file.keys() {
        all_files.insert(f.clone(), ());
    }
    for f in rust_by_file.keys() {
        all_files.insert(f.clone(), ());
    }

    let mut totals = Totals {
        files_scanned,
        files_with_diags: all_files.len(),
        perl_diags: perl_by_file.values().map(std::vec::Vec::len).sum(),
        rust_diags: rust_by_file.values().map(std::vec::Vec::len).sum(),
        ..Totals::default()
    };

    let mut missing_msgs: HashMap<String, usize> = HashMap::new();
    let mut extra_msgs: HashMap<String, usize> = HashMap::new();
    let mut mismatch_examples = Vec::new();
    let mut missing_examples = Vec::new();
    let mut extra_examples = Vec::new();

    for file in all_files.keys() {
        let perl_list = perl_by_file.get(file).cloned().unwrap_or_default();
        let rust_list = rust_by_file.get(file).cloned().unwrap_or_default();

        let mut perl_counts: HashMap<Key, usize> = HashMap::new();
        let mut rust_counts: HashMap<Key, usize> = HashMap::new();

        for d in &perl_list {
            let k = Key {
                line: d.line,
                severity: d.severity,
                message: d.message.clone(),
            };
            *perl_counts.entry(k).or_insert(0) += 1;
        }

        for d in &rust_list {
            let k = Key {
                line: d.line,
                severity: d.severity,
                message: d.message.clone(),
            };
            *rust_counts.entry(k).or_insert(0) += 1;
        }

        // Exact matches first.
        let perl_keys: Vec<Key> = perl_counts.keys().cloned().collect();
        for k in perl_keys {
            if let Some(&r_count) = rust_counts.get(&k) {
                let p_count = *perl_counts.get(&k).unwrap_or(&0);
                let m = min(p_count, r_count);
                if m > 0 {
                    totals.exact_matches += m;
                    subtract_count(&mut perl_counts, &k, m);
                    subtract_count(&mut rust_counts, &k, m);
                }
            }
        }

        // Severity mismatches: same (line, message), different severity.
        let mut lm_keys: BTreeMap<LmKey, ()> = BTreeMap::new();
        for k in perl_counts.keys() {
            lm_keys.insert(
                LmKey {
                    line: k.line,
                    message: k.message.clone(),
                },
                (),
            );
        }
        for k in rust_counts.keys() {
            lm_keys.insert(
                LmKey {
                    line: k.line,
                    message: k.message.clone(),
                },
                (),
            );
        }

        for lm in lm_keys.keys() {
            let p_err = *perl_counts
                .get(&Key {
                    line: lm.line,
                    severity: Severity::Error,
                    message: lm.message.clone(),
                })
                .unwrap_or(&0);
            let p_warn = *perl_counts
                .get(&Key {
                    line: lm.line,
                    severity: Severity::Warning,
                    message: lm.message.clone(),
                })
                .unwrap_or(&0);
            let r_err = *rust_counts
                .get(&Key {
                    line: lm.line,
                    severity: Severity::Error,
                    message: lm.message.clone(),
                })
                .unwrap_or(&0);
            let r_warn = *rust_counts
                .get(&Key {
                    line: lm.line,
                    severity: Severity::Warning,
                    message: lm.message.clone(),
                })
                .unwrap_or(&0);

            let e_to_w = min(p_err, r_warn);
            if e_to_w > 0 {
                totals.severity_mismatches += e_to_w;
                subtract_count(
                    &mut perl_counts,
                    &Key {
                        line: lm.line,
                        severity: Severity::Error,
                        message: lm.message.clone(),
                    },
                    e_to_w,
                );
                subtract_count(
                    &mut rust_counts,
                    &Key {
                        line: lm.line,
                        severity: Severity::Warning,
                        message: lm.message.clone(),
                    },
                    e_to_w,
                );
                if mismatch_examples.len() < 20 {
                    mismatch_examples.push(format!(
                        "{}:{}: checkstyle=ERROR rust=WARNING: {}",
                        file, lm.line, lm.message
                    ));
                }
            }

            let w_to_e = min(p_warn, r_err);
            if w_to_e > 0 {
                totals.severity_mismatches += w_to_e;
                subtract_count(
                    &mut perl_counts,
                    &Key {
                        line: lm.line,
                        severity: Severity::Warning,
                        message: lm.message.clone(),
                    },
                    w_to_e,
                );
                subtract_count(
                    &mut rust_counts,
                    &Key {
                        line: lm.line,
                        severity: Severity::Error,
                        message: lm.message.clone(),
                    },
                    w_to_e,
                );
                if mismatch_examples.len() < 20 {
                    mismatch_examples.push(format!(
                        "{}:{}: checkstyle=WARNING rust=ERROR: {}",
                        file, lm.line, lm.message
                    ));
                }
            }
        }

        for (k, cnt) in &perl_counts {
            totals.missing_in_rust += *cnt;
            *missing_msgs.entry(k.message.clone()).or_insert(0) += *cnt;
            if missing_examples.len() < 20 {
                missing_examples.push(format!(
                    "{}:{}: {}: {}",
                    file,
                    k.line,
                    k.severity.as_str(),
                    k.message
                ));
            }
        }

        for (k, cnt) in &rust_counts {
            totals.extra_in_rust += *cnt;
            *extra_msgs.entry(k.message.clone()).or_insert(0) += *cnt;
            if extra_examples.len() < 20 {
                extra_examples.push(format!(
                    "{}:{}: {}: {}",
                    file,
                    k.line,
                    k.severity.as_str(),
                    k.message
                ));
            }
        }
    }

    CompareReport {
        totals,
        top_missing: top_n(missing_msgs, 15),
        top_extra: top_n(extra_msgs, 15),
        mismatch_examples,
        missing_examples,
        extra_examples,
    }
}

fn subtract_count(map: &mut HashMap<Key, usize>, key: &Key, by: usize) {
    if let Some(v) = map.get_mut(key) {
        if *v <= by {
            map.remove(key);
        } else {
            *v -= by;
        }
    }
}

fn top_n(map: HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
    let mut v: Vec<(String, usize)> = map.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

fn print_report(r: &CompareReport) {
    println!("Checkstyle Parity Matrix");
    println!("- Files scanned: {}", r.totals.files_scanned);
    println!("- Files with diagnostics: {}", r.totals.files_with_diags);
    println!("- checkstyle diagnostics: {}", r.totals.perl_diags);
    println!("- rust diagnostics: {}", r.totals.rust_diags);
    println!("- Exact matches: {}", r.totals.exact_matches);
    println!("- Severity mismatches: {}", r.totals.severity_mismatches);
    println!("- Missing in Rust: {}", r.totals.missing_in_rust);
    println!("- Extra in Rust: {}", r.totals.extra_in_rust);

    if !r.top_missing.is_empty() {
        println!();
        println!("Top missing messages (checkstyle -> not produced by rust):");
        for (msg, n) in &r.top_missing {
            println!("- {:>5}  {}", n, msg);
        }
    }

    if !r.top_extra.is_empty() {
        println!();
        println!("Top extra messages (rust -> not produced by checkstyle):");
        for (msg, n) in &r.top_extra {
            println!("- {:>5}  {}", n, msg);
        }
    }

    if !r.mismatch_examples.is_empty() {
        println!();
        println!("Severity mismatch examples:");
        for item in &r.mismatch_examples {
            println!("- {}", item);
        }
    }

    if !r.missing_examples.is_empty() {
        println!();
        println!("Missing examples:");
        for item in &r.missing_examples {
            println!("- {}", item);
        }
    }

    if !r.extra_examples.is_empty() {
        println!();
        println!("Extra examples:");
        for item in &r.extra_examples {
            println!("- {}", item);
        }
    }
}
