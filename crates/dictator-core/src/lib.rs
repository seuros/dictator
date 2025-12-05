#![warn(rust_2024_compatibility, clippy::all)]

pub mod config;
pub mod linter_output;

use anyhow::Result;
use camino::Utf8Path;
use dictator_decree_abi::{BoxDecree, Diagnostics};
use std::collections::HashSet;

pub use config::{DecreeSettings, DictateConfig};

/// In-memory source file for the Regime to enforce.
pub struct Source<'a> {
    pub path: &'a Utf8Path,
    pub text: &'a str,
}

/// The Regime: owns decree instances and enforces them over sources.
pub struct Regime {
    decrees: Vec<BoxDecree>,
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
    pub fn add_wasm_decree<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let decree = loader::load_decree(path.as_ref())?;
        self.decrees.push(decree);
        Ok(())
    }

    /// Enforce all decrees over provided sources.
    ///
    /// Only runs a decree on files whose extension matches the decree's
    /// `supported_extensions`. Decrees with empty `supported_extensions`
    /// (like decree.supreme) run on all files.
    ///
    /// # Errors
    ///
    /// Returns an error if any decree fails during linting.
    pub fn enforce(&self, sources: &[Source<'_>]) -> Result<Diagnostics> {
        let mut all = Diagnostics::new();
        for decree in &self.decrees {
            let supported = &decree.metadata().supported_extensions;
            for src in sources {
                // Empty supported_extensions means "all files" (e.g., decree.supreme)
                if supported.is_empty() || Self::extension_matches(src.path, supported) {
                    all.extend(decree.lint(src.path.as_str(), src.text));
                }
            }
        }
        Ok(all)
    }

    /// Check if a file's extension matches any in the supported list.
    fn extension_matches(path: &Utf8Path, supported: &[String]) -> bool {
        path.extension()
            .is_some_and(|ext| supported.iter().any(|s| s == ext))
    }
}

mod loader {
    use anyhow::{Context, Result};
    use dictator_decree_abi::{BoxDecree, Diagnostics, Span};
    use libloading::Library;
    use std::path::Path;
    use std::sync::Mutex;
    use wasmtime::component::{Component, Linker, ResourceTable};
    use wasmtime::{Config, Engine, Store};
    use wasmtime_wasi::p2::add_to_linker_sync;
    use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

    mod bindings {
        wasmtime::component::bindgen!({ path: "wit/decree.wit", world: "decree" });
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

        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;
        let component = Component::from_file(&engine, lib_path)
            .with_context(|| format!("failed to load wasm decree: {}", lib_path.display()))?;
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
            .context("failed to call metadata on wasm decree")?;

        let metadata = dictator_decree_abi::DecreeMetadata {
            abi_version: wasm_meta.abi_version,
            decree_version: wasm_meta.decree_version,
            description: wasm_meta.description,
            dectauthors: wasm_meta.dectauthors,
            supported_extensions: wasm_meta.supported_extensions,
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

        metadata
            .validate_abi(ABI_VERSION)
            .map_err(|e| anyhow::anyhow!("Decree '{}' from {}: {}", name, lib_path.display(), e))?;

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
mod tests {
    use super::*;
    use dictator_decree_abi::{Capability, Decree, DecreeMetadata, Diagnostics};

    struct MockDecree {
        name: &'static str,
        exts: Vec<String>,
    }

    impl Decree for MockDecree {
        fn name(&self) -> &str {
            self.name
        }

        fn lint(&self, _path: &str, _source: &str) -> Diagnostics {
            Diagnostics::new()
        }

        fn metadata(&self) -> DecreeMetadata {
            DecreeMetadata {
                abi_version: "1".into(),
                decree_version: "1".into(),
                description: String::new(),
                dectauthors: None,
                supported_extensions: self.exts.clone(),
                capabilities: vec![Capability::Lint],
            }
        }
    }

    #[test]
    fn watched_extensions_unites_declared_sets() {
        let decree_a: BoxDecree = Box::new(MockDecree {
            name: "a",
            exts: vec!["rs".into(), "Rb".into()],
        });
        let decree_b: BoxDecree = Box::new(MockDecree {
            name: "b",
            exts: vec!["ts".into()],
        });
        let mut regime = Regime::new();
        regime.add_decree(decree_a);
        regime.add_decree(decree_b);

        let exts = regime.watched_extensions().unwrap();
        assert!(exts.contains("rs"));
        assert!(exts.contains("rb"));
        assert!(exts.contains("ts"));
        assert_eq!(exts.len(), 3);
    }

    #[test]
    fn watched_extensions_none_when_only_universal() {
        let sup: BoxDecree = Box::new(MockDecree {
            name: "supreme",
            exts: vec![], // universal decree declares no extensions
        });
        let mut regime = Regime::new();
        regime.add_decree(sup);

        assert!(regime.watched_extensions().is_none());
    }
}
