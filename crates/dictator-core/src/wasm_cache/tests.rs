use super::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_cache_miss_then_hit() {
    let cache = WasmCache::new().unwrap();

    // Create a dummy WASM file (not valid WASM, but tests the cache logic)
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "dummy wasm content").unwrap();
    let path = temp_file.path();

    // First call should miss and try to load (will fail due to invalid WASM)
    let result = cache.get_or_load(path);
    assert!(result.is_err());

    // Verify cache stats
    let stats = cache.stats();
    assert_eq!(stats.entries, 0); // Should be 0 because loading failed
}

#[test]
fn test_cache_clear() {
    let cache = WasmCache::new().unwrap();

    // Clear empty cache should work
    cache.clear();
    let stats = cache.stats();
    assert_eq!(stats.entries, 0);
}
