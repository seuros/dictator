//! WASM module caching for improved performance

use anyhow::{Context, Result};
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use wasmtime::component::Component;
use wasmtime::{Config, Engine};

/// Cache for compiled WASM components
pub struct WasmCache {
    engine: Engine,
    components: DashMap<PathBuf, Arc<Component>>,
}

impl WasmCache {
    /// Create a new WASM cache with component model enabled
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;

        Ok(Self {
            engine,
            components: DashMap::new(),
        })
    }

    /// Get a compiled component from cache or load and compile it
    pub fn get_or_load(&self, path: &Path) -> Result<Arc<Component>> {
        // Check if component is already cached
        if let Some(component) = self.components.get(path) {
            return Ok::<Arc<Component>, anyhow::Error>(component.clone());
        }

        // Load and compile the component
        let component = Component::from_file(&self.engine, path)
            .map_err(anyhow::Error::from)
            .with_context(|| format!("failed to load wasm decree: {}", path.display()))?;

        let component = Arc::new(component);

        // Cache the compiled component
        self.components
            .insert(path.to_path_buf(), component.clone());

        tracing::info!("Cached WASM component: {}", path.display());
        Ok(component)
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.components.clear();
        tracing::info!("Cleared WASM component cache");
    }

    /// Get cache statistics
    #[must_use]
    pub fn stats(&self) -> WasmCacheStats {
        WasmCacheStats {
            entries: self.components.len(),
        }
    }
}

impl Default for WasmCache {
    fn default() -> Self {
        Self::new().expect("Failed to create WASM cache")
    }
}

/// Statistics about the WASM cache
#[derive(Debug, Clone)]
pub struct WasmCacheStats {
    pub entries: usize,
}

#[cfg(test)]
mod tests;
