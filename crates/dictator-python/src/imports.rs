//! Import ordering checks for Python sources (PEP 8 compliant).

use dictator_decree_abi::{Diagnostic, Diagnostics, Span};
use memchr::memchr_iter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportType {
    Stdlib,     // Python standard library: os, sys, json, etc.
    ThirdParty, // External packages: requests, django, etc.
    Local,      // Relative imports: . or ..
}

impl ImportType {
    const fn order(self) -> u8 {
        match self {
            Self::Stdlib => 0,
            Self::ThirdParty => 1,
            Self::Local => 2,
        }
    }
}

pub fn check_import_ordering(source: &str, diags: &mut Diagnostics) {
    let bytes = source.as_bytes();
    let mut imports: Vec<(usize, usize, ImportType)> = Vec::new();
    let mut line_start = 0;

    for nl in memchr_iter(b'\n', bytes) {
        let line = &source[line_start..nl];
        let trimmed = line.trim();

        if let Some(import_type) = parse_import_line(trimmed) {
            imports.push((line_start, nl, import_type));
        }

        // Stop at first non-import, non-comment, non-blank, non-docstring line
        if !trimmed.is_empty()
            && !trimmed.starts_with("import")
            && !trimmed.starts_with("from")
            && !trimmed.starts_with('#')
            && !trimmed.starts_with("\"\"\"")
            && !trimmed.starts_with("'''")
            && !trimmed.ends_with("\"\"\"")
            && !trimmed.ends_with("'''")
            && !trimmed.starts_with("__future__")
        {
            break;
        }

        line_start = nl + 1;
    }

    // Check import order
    if imports.len() > 1 {
        let mut last_type = ImportType::Stdlib;

        for (start, end, import_type) in &imports {
            if import_type.order() < last_type.order() {
                diags.push(Diagnostic {
                    rule: "python/import-order".to_string(),
                    message: format!(
                        "Import order violation: {import_type:?} import after {last_type:?} import. Expected order: stdlib -> third_party -> local"
                    ),
                    enforced: false,
                    span: Span::new(*start, *end),
                });
            }

            last_type = *import_type;
        }
    }
}

fn parse_import_line(line: &str) -> Option<ImportType> {
    if !line.starts_with("import") && !line.starts_with("from") {
        return None;
    }

    // Handle "from X import Y" style
    if line.starts_with("from") {
        let from_keyword = "from ";
        if let Some(pos) = line.find(from_keyword) {
            let after_from = &line[pos + from_keyword.len()..];
            let module_name = after_from.split_whitespace().next()?.trim_end_matches(',');

            return Some(classify_module(module_name));
        }
    }

    // Handle "import X" style
    if line.starts_with("import") {
        let import_keyword = "import ";
        if let Some(pos) = line.find(import_keyword) {
            let after_import = &line[pos + import_keyword.len()..];
            let module_name = after_import
                .split([',', ';'])
                .next()?
                .split_whitespace()
                .next()?
                .trim_end_matches(',');

            return Some(classify_module(module_name));
        }
    }

    None
}

#[must_use]
pub fn classify_module(module_name: &str) -> ImportType {
    // Local imports start with . or ..
    if module_name.starts_with('.') {
        return ImportType::Local;
    }

    // Get the top-level package name
    let top_level = module_name.split('.').next().unwrap_or(module_name);

    if is_python_stdlib(top_level) {
        ImportType::Stdlib
    } else {
        ImportType::ThirdParty
    }
}

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn is_python_stdlib(module: &str) -> bool {
    matches!(
        module,
        "__future__"
            | "__main__"
            | "abc"
            | "argparse"
            | "array"
            | "ast"
            | "asyncio"
            | "atexit"
            | "base64"
            | "bisect"
            | "builtins"
            | "bz2"
            | "calendar"
            | "cmath"
            | "cmd"
            | "code"
            | "codecs"
            | "collections"
            | "concurrent"
            | "configparser"
            | "contextlib"
            | "contextvars"
            | "copy"
            | "copyreg"
            | "csv"
            | "ctypes"
            | "curses"
            | "dataclasses"
            | "datetime"
            | "dbm"
            | "decimal"
            | "difflib"
            | "dis"
            | "distutils"
            | "doctest"
            | "email"
            | "encodings"
            | "enum"
            | "errno"
            | "fcntl"
            | "filecmp"
            | "fileinput"
            | "fnmatch"
            | "fractions"
            | "functools"
            | "gc"
            | "getopt"
            | "getpass"
            | "gettext"
            | "glob"
            | "gzip"
            | "hashlib"
            | "heapq"
            | "hmac"
            | "html"
            | "http"
            | "importlib"
            | "inspect"
            | "io"
            | "ipaddress"
            | "itertools"
            | "json"
            | "keyword"
            | "locale"
            | "logging"
            | "lzma"
            | "marshal"
            | "math"
            | "mimetypes"
            | "mmap"
            | "multiprocessing"
            | "numbers"
            | "operator"
            | "optparse"
            | "os"
            | "pathlib"
            | "pdb"
            | "pickle"
            | "pipes"
            | "pkgutil"
            | "platform"
            | "pprint"
            | "profile"
            | "pstats"
            | "pwd"
            | "py_compile"
            | "pydoc"
            | "queue"
            | "random"
            | "re"
            | "readline"
            | "reprlib"
            | "resource"
            | "runpy"
            | "sched"
            | "secrets"
            | "select"
            | "selectors"
            | "shelve"
            | "shlex"
            | "shutil"
            | "signal"
            | "site"
            | "smtplib"
            | "socket"
            | "sqlite3"
            | "ssl"
            | "stat"
            | "statistics"
            | "string"
            | "struct"
            | "subprocess"
            | "sys"
            | "sysconfig"
            | "syslog"
            | "tarfile"
            | "tempfile"
            | "test"
            | "textwrap"
            | "threading"
            | "time"
            | "timeit"
            | "tkinter"
            | "token"
            | "tokenize"
            | "trace"
            | "traceback"
            | "tracemalloc"
            | "tty"
            | "turtle"
            | "types"
            | "typing"
            | "typing_extensions"
            | "unittest"
            | "urllib"
            | "uuid"
            | "venv"
            | "warnings"
            | "wave"
            | "weakref"
            | "webbrowser"
            | "xml"
            | "xmlrpc"
            | "zipfile"
            | "zlib"
    )
}
