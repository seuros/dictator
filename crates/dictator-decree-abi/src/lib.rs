#![warn(rust_2024_compatibility, clippy::all)]

use serde::{Deserialize, Serialize};

/// ABI version for decree compatibility checking.
///
/// Bumped when Plugin trait or core types change.
/// Pre-1.0: exact major.minor match required (0.1.x ↔ 0.1.y ✓, 0.1.x ↔ 0.2.y ✗)
/// Post-1.0: major must match, decree minor ≤ host minor
pub const ABI_VERSION: &str = "0.1.0";

/// Capability flags for decrees
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability {
    /// Basic linting (always required)
    Lint,
    /// Can auto-fix issues
    AutoFix,
    /// Supports incremental/streaming linting
    Streaming,
    /// Accepts config at lint-time
    RuntimeConfig,
    /// Returns enhanced diagnostics (quickfixes, etc.)
    RichDiagnostics,
}

/// Metadata for decree versioning and capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecreeMetadata {
    /// ABI version this decree was built against
    pub abi_version: String,
    /// Decree's own version (e.g., "0.60.0" for kjr)
    pub decree_version: String,
    /// Human-readable description
    pub description: String,
    /// Decree authors (from workspace, optional)
    pub dectauthors: Option<String>,
    /// File extensions this decree handles (e.g., `["rb", "rake"]`)
    pub supported_extensions: Vec<String>,
    /// Capabilities this decree provides
    pub capabilities: Vec<Capability>,
}

impl DecreeMetadata {
    /// Check if this decree has a specific capability.
    #[must_use]
    pub fn has_capability(&self, cap: Capability) -> bool {
        self.capabilities.contains(&cap)
    }

    /// Parse semver version string.
    ///
    /// # Errors
    ///
    /// Returns an error if the version string is not in the format "major.minor.patch"
    /// or if any component cannot be parsed as a u32.
    pub fn parse_version(version: &str) -> Result<(u32, u32, u32), String> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("invalid version format: {version}"));
        }
        let major = parts[0]
            .parse()
            .map_err(|_| format!("invalid major: {}", parts[0]))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| format!("invalid minor: {}", parts[1]))?;
        let patch = parts[2]
            .parse()
            .map_err(|_| format!("invalid patch: {}", parts[2]))?;
        Ok((major, minor, patch))
    }

    /// Check if this decree's ABI version is compatible with host ABI version.
    ///
    /// # Errors
    ///
    /// Returns an error if the ABI versions are incompatible or if version parsing fails.
    pub fn validate_abi(&self, host_abi_version: &str) -> Result<(), String> {
        let (host_maj, host_min, _) = Self::parse_version(host_abi_version)?;
        let (decree_maj, decree_min, _) = Self::parse_version(&self.abi_version)?;

        // Pre-1.0: exact major.minor match required
        if host_maj == 0 {
            if host_maj == decree_maj && host_min == decree_min {
                return Ok(());
            }
            return Err(format!(
                "ABI version mismatch: host {}, decree {}",
                host_abi_version, self.abi_version
            ));
        }

        // Post-1.0: major must match, decree minor ≤ host minor
        if host_maj == decree_maj && decree_min <= host_min {
            return Ok(());
        }

        Err(format!(
            "ABI version incompatible: host {}, decree {}",
            host_abi_version, self.abi_version
        ))
    }
}

/// Byte offsets into the source file (half-open range).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    /// Create a new span with the given start and end offsets.
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Check if this span is empty (start >= end).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Rule identifier, e.g. "ruby/trailing-whitespace".
    pub rule: String,
    pub message: String,
    pub span: Span,
    /// true = Dictator enforced (auto-fixed), false = you must comply
    pub enforced: bool,
}

pub type Diagnostics = Vec<Diagnostic>;

/// Trait all Dictator decrees must implement. Designed to be usable from WASM by
/// exporting a thin C-ABI shim that instantiates a concrete implementor.
pub trait Decree: Send + Sync {
    /// Human-friendly decree name, e.g. "ruby".
    #[must_use]
    fn name(&self) -> &str;

    /// Lint a single file, returning diagnostics. `path` is UTF-8.
    fn lint(&self, path: &str, source: &str) -> Diagnostics;

    /// Metadata for versioning and capabilities.
    #[must_use]
    fn metadata(&self) -> DecreeMetadata;

    /// Create rule identifier: `{decree}/{rule}` - DRY helper.
    #[must_use]
    fn rule(&self, rule_name: &str) -> String {
        format!("{}/{}", self.name(), rule_name)
    }
}

/// Boxed decree for dynamic dispatch.
pub type BoxDecree = Box<dyn Decree>;

/// Function exported by WASM decrees to construct an instance.
pub type DecreeFactory = fn() -> BoxDecree;

/// Export name expected in decrees for the factory symbol.
pub const DECREE_FACTORY_EXPORT: &str = "dictator_create_decree";
