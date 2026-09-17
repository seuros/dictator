#![warn(rust_2024_compatibility, clippy::all)]

pub mod classified;
pub mod config;
mod decree_matching;
pub mod error;
pub mod linter_output;
mod rule_ignoring;
pub mod wasm_cache;

use anyhow::Result;
use camino::Utf8Path;
use dictator_decree_abi::{BoxDecree, Diagnostic, Diagnostics, Span};
use std::collections::{HashMap, HashSet};

pub use config::{DecreeSettings, DictateConfig};
pub use error::{DictatorContext, DictatorError, suggestions};
pub use wasm_cache::WasmCacheStats;

/// In-memory source file for the Regime to enforce.
pub struct Source<'a> {
    pub path: &'a Utf8Path,
    pub text: &'a str,
}

/// The Regime: owns decree instances and enforces them over sources.
pub struct Regime {
    decrees: Vec<BoxDecree>,
    rule_ignores: rule_ignoring::RuleIgnores,
}

impl Default for Regime {
    fn default() -> Self {
        Self::new()
    }
}

impl Regime {
    #[must_use]
    pub fn new() -> Self {
        Self {
            decrees: Vec::new(),
            rule_ignores: rule_ignoring::RuleIgnores::new(),
        }
    }

    /// Get WASM cache statistics if available
    #[must_use]
    pub fn wasm_cache_stats(&self) -> Option<WasmCacheStats> {
        #[cfg(feature = "wasm-loader")]
        {
            use loader::get_wasm_engine_cache;
            let (_, cache) = get_wasm_engine_cache();
            Some(cache.stats())
        }
        #[cfg(not(feature = "wasm-loader"))]
        {
            let _ = self; // suppress unused variable warning
            None
        }
    }

    #[must_use]
    pub fn with_decree(mut self, decree: BoxDecree) -> Self {
        self.decrees.push(decree);
        self
    }

    pub fn add_decree(&mut self, decree: BoxDecree) {
        self.decrees.push(decree);
    }

    /// Configure per-rule ignores from a loaded `.dictate.toml`.
    pub fn set_rule_ignores_from_config(&mut self, config: Option<&DictateConfig>) {
        self.rule_ignores = rule_ignoring::build_rule_ignores(config);
    }

    /// Map each loaded decree's name to its persona (e.g. `"freebsd"` ->
    /// `"Beastie"`), for persona-voiced diagnostic summaries.
    #[must_use]
    pub fn personas(&self) -> HashMap<String, String> {
        self.decrees
            .iter()
            .map(|d| (d.name().to_string(), d.metadata().persona))
            .collect()
    }

    /// Union of every loaded decree's file-scope rule names (bare, unprefixed).
    #[must_use]
    pub fn file_scope_rules(&self) -> HashSet<String> {
        self.decrees
            .iter()
            .flat_map(|d| d.metadata().file_scope_rules)
            .collect()
    }

    /// Return the union of supported extensions for all loaded decrees.
    ///
    /// - If at least one decree declares specific extensions, returns `Some(HashSet)` of
    ///   those (lowercased) extensions.
    /// - If no decree declares extensions (all empty lists), returns `None`, meaning
    ///   "watch everything" (typical when only supreme is loaded).
    #[must_use]
    pub fn watched_extensions(&self) -> Option<HashSet<String>> {
        let mut exts = HashSet::new();
        for decree in &self.decrees {
            let supported = &decree.metadata().supported_extensions;
            if supported.is_empty() {
                continue; // empty means "all" for enforcement, but we don't widen the watch set
            }
            for ext in supported {
                exts.insert(ext.to_ascii_lowercase());
            }
        }

        if exts.is_empty() { None } else { Some(exts) }
    }

    /// Load a WASM decree from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be loaded, if it's not a valid WASM/native decree,
    /// or if the decree's ABI version is incompatible with the host.
    #[cfg(feature = "wasm-loader")]
    pub fn add_wasm_decree<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let decree = loader::load_decree(path.as_ref())?;
        self.decrees.push(decree);
        Ok(())
    }

    #[cfg(not(feature = "wasm-loader"))]
    pub fn add_wasm_decree<P: AsRef<std::path::Path>>(&mut self, _path: P) -> Result<()> {
        anyhow::bail!("WASM loader disabled; enable the `wasm-loader` feature to load decrees");
    }

    /// Enforce all decrees over provided sources.
    ///
    /// Matching priority:
    /// 1. `skip_filenames` - decree owns file but returns empty diagnostics
    /// 2. `supported_filenames` - exact filename match
    /// 3. `supported_extensions` - extension match
    /// 4. Universal decrees (empty lists) run on all files unless shadowed
    ///
    /// # Errors
    ///
    /// Returns an error if any decree fails during linting.
    pub fn enforce(&self, sources: &[Source<'_>]) -> Result<Diagnostics> {
        let mut all = Diagnostics::new();
        for src in sources {
            // Classified files get one diagnostic and no inspection or fixes
            if classified::is_classified(src.path) {
                all.push(Diagnostic {
                    rule: classified::CLASSIFIED_RULE.to_string(),
                    message: "classified material — contents exempt from inspection".to_string(),
                    span: Span { start: 0, end: 0 },
                    enforced: false,
                });
                continue;
            }

            let filename = src.path.file_name().unwrap_or("");

            // CSS-style specificity: if a language-specific decree is present for this file
            // type, do not run the catch-all decree.supreme on this file.
            let is_supreme_shadowed = decree_matching::is_supreme_shadowed(&self.decrees, src.path);

            for decree in &self.decrees {
                let meta = decree.metadata();

                // Skip files in skip_filenames (owned but not linted)
                if meta.skip_filenames.iter().any(|s| s == filename) {
                    continue;
                }

                // Check if decree matches this file
                let matches = decree_matching::decree_matches(src.path, &meta);
                if !matches {
                    continue;
                }

                // Universal decrees shadowed by language-specific ones
                let is_universal =
                    meta.supported_extensions.is_empty() && meta.supported_filenames.is_empty();
                if is_supreme_shadowed && is_universal && decree.name() == "supreme" {
                    continue;
                }

                let diags = decree.lint(src.path.as_str(), src.text);
                for diag in diags {
                    let ignores = &self.rule_ignores;
                    if rule_ignoring::is_rule_ignored_for_path(ignores, src.path, &diag) {
                        continue;
                    }
                    all.push(diag);
                }
            }
        }
        Ok(all)
    }
}

#[cfg(feature = "wasm-loader")]
pub(crate) mod loader {
    use crate::wasm_cache::WasmCache;
    use anyhow::{Context, Result};
    use dictator_decree_abi::{BoxDecree, Diagnostics, Span};
    use libloading::Library;
    use std::path::Path;
    use std::sync::{Arc, Mutex, OnceLock};

    use wasmtime::component::{Linker, ResourceTable};
    use wasmtime::{Engine, Store};
    use wasmtime_wasi::p2::add_to_linker_sync;
    use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

    mod bindings {
        wasmtime::component::bindgen!({ path: "wit/decree.wit", world: "decree" });
    }

    /// Global WASM engine and cache instance
    static WASM_ENGINE_CACHE: OnceLock<(Engine, Arc<WasmCache>)> = OnceLock::new();

    /// Get or create the global WASM engine and cache
    pub fn get_wasm_engine_cache() -> (Engine, Arc<WasmCache>) {
        WASM_ENGINE_CACHE
            .get_or_init(|| {
                let mut config = wasmtime::Config::new();
                config.wasm_component_model(true);
                let engine = Engine::new(&config).expect("Failed to create WASM engine");
                let cache = Arc::new(WasmCache::new().expect("Failed to create WASM cache"));
                (engine, cache)
            })
            .clone()
    }

    /// Load a decree compiled as a native dynamic library (.dylib/.so/.dll).
    ///
    /// # Safety
    /// Loading dynamic libraries is inherently unsafe. The library must:
    /// - Export a valid `dictator_create_decree` symbol
    /// - Return a valid boxed Decree
    /// - Not cause undefined behavior when called
    #[allow(unsafe_code)]
    fn load_native(lib_path: &Path) -> Result<BoxDecree> {
        use dictator_decree_abi::{ABI_VERSION, DECREE_FACTORY_EXPORT, DecreeFactory};

        // We must keep the library handle alive for the lifetime of the process; unloading
        // invalidates function pointers held by the decree and triggers UB. We keep every
        // successfully loaded Library in a global registry instead of letting it drop.
        static LOADED_LIBRARIES: std::sync::OnceLock<std::sync::Mutex<Vec<Library>>> =
            std::sync::OnceLock::new();

        unsafe {
            let lib = Library::new(lib_path)
                .with_context(|| format!("failed to load native decree: {}", lib_path.display()))?;
            let ctor: libloading::Symbol<DecreeFactory> =
                lib.get(DECREE_FACTORY_EXPORT.as_bytes()).with_context(|| {
                    format!(
                        "missing symbol {} in {}",
                        DECREE_FACTORY_EXPORT,
                        lib_path.display()
                    )
                })?;

            let decree = ctor();

            // Validate ABI compatibility
            let metadata = decree.metadata();
            metadata.validate_abi(ABI_VERSION).map_err(|e| {
                anyhow::anyhow!(
                    "Decree '{}' from {}: {}",
                    decree.name(),
                    lib_path.display(),
                    e
                )
            })?;

            tracing::info!(
                "Loaded decree '{}' v{} (ABI {})",
                decree.name(),
                metadata.decree_version,
                metadata.abi_version
            );

            // Park the library handle so it is never dropped/unloaded.
            LOADED_LIBRARIES
                .get_or_init(std::sync::Mutex::default)
                .lock()
                .expect("loaded libraries mutex poisoned")
                .push(lib);

            Ok(decree)
        }
    }

    use self::bindings::exports::dictator::decree::lints as guest;

    struct HostState {
        table: ResourceTable,
        wasi: WasiCtx,
    }

    impl WasiView for HostState {
        fn ctx(&mut self) -> WasiCtxView<'_> {
            WasiCtxView {
                ctx: &mut self.wasi,
                table: &mut self.table,
            }
        }
    }

    struct WasmDecree {
        name: String,
        metadata: dictator_decree_abi::DecreeMetadata,
        state: Mutex<WasmState>,
    }

    struct WasmState {
        store: Store<HostState>,
        plugin: bindings::Decree,
    }

    impl dictator_decree_abi::Decree for WasmDecree {
        fn name(&self) -> &str {
            &self.name
        }

        #[allow(clippy::significant_drop_tightening)]
        fn lint(&self, path: &str, source: &str) -> Diagnostics {
            let result = {
                let mut guard = self.state.lock().expect("wasm store poisoned");
                let WasmState { plugin, store } = &mut *guard;
                plugin
                    .dictator_decree_lints()
                    .call_lint(store, path, source)
                    .unwrap_or_default()
            };
            result
                .into_iter()
                .map(|d| dictator_decree_abi::Diagnostic {
                    rule: d.rule,
                    message: d.message,
                    enforced: matches!(d.severity, guest::Severity::Info), // Info = auto-fixed
                    span: Span {
                        start: d.span.start as usize,
                        end: d.span.end as usize,
                    },
                })
                .collect()
        }

        fn metadata(&self) -> dictator_decree_abi::DecreeMetadata {
            self.metadata.clone()
        }
    }

    fn load_wasm(lib_path: &Path) -> Result<BoxDecree> {
        use dictator_decree_abi::ABI_VERSION;

        let (engine, cache) = get_wasm_engine_cache();
        let component = cache.get_or_load(lib_path).with_context(|| {
            format!("failed to load cached wasm decree: {}", lib_path.display())
        })?;

        let mut linker: Linker<HostState> = Linker::new(&engine);
        add_to_linker_sync(&mut linker)?;
        let host_state = HostState {
            table: ResourceTable::new(),
            wasi: WasiCtxBuilder::new().inherit_stdio().build(),
        };
        let mut store = Store::new(&engine, host_state);
        let plugin = bindings::Decree::instantiate(&mut store, &component, &linker)?;
        let guest = plugin.dictator_decree_lints();

        let name = guest
            .call_name(&mut store)
            .unwrap_or_else(|_| "wasm-decree".to_string());

        // Get and validate metadata
        let wasm_meta = guest
            .call_metadata(&mut store)
            .map_err(anyhow::Error::from)
            .context("failed to call metadata on wasm decree")?;

        let metadata = dictator_decree_abi::DecreeMetadata {
            abi_version: wasm_meta.abi_version,
            decree_version: wasm_meta.decree_version,
            description: wasm_meta.description,
            persona: wasm_meta.persona,
            dectauthors: wasm_meta.dectauthors,
            supported_extensions: wasm_meta.supported_extensions,
            supported_filenames: wasm_meta.supported_filenames,
            skip_filenames: wasm_meta.skip_filenames,
            file_scope_rules: wasm_meta.file_scope_rules,
            capabilities: wasm_meta
                .capabilities
                .into_iter()
                .map(|c| match c {
                    guest::Capability::Lint => dictator_decree_abi::Capability::Lint,
                    guest::Capability::AutoFix => dictator_decree_abi::Capability::AutoFix,
                    guest::Capability::Streaming => dictator_decree_abi::Capability::Streaming,
                    guest::Capability::RuntimeConfig => {
                        dictator_decree_abi::Capability::RuntimeConfig
                    }
                    guest::Capability::RichDiagnostics => {
                        dictator_decree_abi::Capability::RichDiagnostics
                    }
                })
                .collect(),
        };

        metadata.validate_abi(ABI_VERSION).map_err(|_e| {
            let path = lib_path.to_path_buf();
            crate::DictatorError::WasmLoadError {
                path,
                abi_version: metadata.abi_version.clone(),
                expected: ABI_VERSION.to_string(),
                suggestion: crate::suggestions::wasm_suggestions("abi_mismatch").to_string(),
            }
        })?;

        tracing::info!(
            "Loaded WASM decree '{}' v{} (ABI {})",
            name,
            metadata.decree_version,
            metadata.abi_version
        );

        Ok(Box::new(WasmDecree {
            name,
            metadata,
            state: Mutex::new(WasmState { store, plugin }),
        }))
    }

    pub fn load_decree(path: &Path) -> Result<BoxDecree> {
        match path.extension().and_then(|s| s.to_str()) {
            Some("wasm") => load_wasm(path),
            _ => load_native(path),
        }
    }
}

#[cfg(test)]
mod tests;
