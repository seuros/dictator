//! Enhanced error context and diagnostics for Dictator

use anyhow::Result;
use std::fmt;
use std::path::PathBuf;

/// Enhanced error types with context and suggestions
#[derive(Debug, Clone)]
pub enum DictatorError {
    /// Configuration-related errors
    ConfigError { file: PathBuf, line: Option<usize>, message: String, suggestion: String },

    /// WASM decree loading errors
    WasmLoadError { path: PathBuf, abi_version: String, expected: String, suggestion: String },

    /// File processing errors
    FileProcessingError { path: PathBuf, operation: String, message: String, suggestion: String },

    /// Rule configuration errors
    RuleConfigurationError { decree: String, rule: String, message: String, suggestion: String },

    /// Performance-related warnings
    PerformanceWarning { context: String, message: String, suggestion: String },
}

impl fmt::Display for DictatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigError { file, line, message, suggestion } => {
                write!(f, "Configuration error")?;
                if let Some(line_num) = line {
                    write!(f, " at {}:{}", file.display(), line_num)?;
                } else {
                    write!(f, " in {}", file.display())?;
                }
                write!(f, ": {message}")?;
                write!(f, "\n💡 Suggestion: {suggestion}")?;
            }
            Self::WasmLoadError { path, abi_version, expected, suggestion } => {
                write!(
                    f,
                    "WASM decree load error for {}: ABI version {} doesn't match expected {}",
                    path.display(),
                    abi_version,
                    expected
                )?;
                write!(f, "\n💡 Suggestion: {suggestion}")?;
            }
            Self::FileProcessingError { path, operation, message, suggestion } => {
                write!(
                    f,
                    "File processing error during '{}' for {}: {}",
                    operation,
                    path.display(),
                    message
                )?;
                write!(f, "\n💡 Suggestion: {suggestion}")?;
            }
            Self::RuleConfigurationError { decree, rule, message, suggestion } => {
                write!(f, "Rule configuration error for {decree}::{rule}: {message}")?;
                write!(f, "\n💡 Suggestion: {suggestion}")?;
            }
            Self::PerformanceWarning { context, message, suggestion } => {
                write!(f, "Performance warning ({context}): {message}")?;
                write!(f, "\n💡 Suggestion: {suggestion}")?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for DictatorError {}

/// Extension trait for adding Dictator-specific context to Results
pub trait DictatorContext<T> {
    /// Add configuration error context
    fn config_context(self, file: PathBuf, line: Option<usize>, suggestion: &str) -> Result<T>;

    /// Add WASM loading error context
    fn wasm_context(
        self,
        path: PathBuf,
        abi_version: &str,
        expected: &str,
        suggestion: &str,
    ) -> Result<T>;

    /// Add file processing error context
    fn file_context(self, path: PathBuf, operation: &str, suggestion: &str) -> Result<T>;

    /// Add rule configuration error context
    fn rule_context(self, decree: &str, rule: &str, suggestion: &str) -> Result<T>;

    /// Add performance warning context
    fn performance_context(self, context: &str, suggestion: &str) -> Result<T>;
}

impl<T, E> DictatorContext<T> for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn config_context(self, file: PathBuf, line: Option<usize>, suggestion: &str) -> Result<T> {
        self.map_err(|e| {
            let error = e.into();
            anyhow::Error::new(DictatorError::ConfigError {
                file,
                line,
                message: error.to_string(),
                suggestion: suggestion.to_string(),
            })
        })
    }

    fn wasm_context(
        self,
        path: PathBuf,
        abi_version: &str,
        expected: &str,
        suggestion: &str,
    ) -> Result<T> {
        self.map_err(|e| {
            // Original error discarded; WASM errors use explicit params
            let _: anyhow::Error = e.into();
            anyhow::Error::new(DictatorError::WasmLoadError {
                path,
                abi_version: abi_version.to_string(),
                expected: expected.to_string(),
                suggestion: suggestion.to_string(),
            })
        })
    }

    fn file_context(self, path: PathBuf, operation: &str, suggestion: &str) -> Result<T> {
        self.map_err(|e| {
            let error: anyhow::Error = e.into();
            anyhow::Error::new(DictatorError::FileProcessingError {
                path,
                operation: operation.to_string(),
                message: error.to_string(),
                suggestion: suggestion.to_string(),
            })
        })
    }

    fn rule_context(self, decree: &str, rule: &str, suggestion: &str) -> Result<T> {
        self.map_err(|e| {
            let error = e.into();
            anyhow::Error::new(DictatorError::RuleConfigurationError {
                decree: decree.to_string(),
                rule: rule.to_string(),
                message: error.to_string(),
                suggestion: suggestion.to_string(),
            })
        })
    }

    fn performance_context(self, context: &str, suggestion: &str) -> Result<T> {
        self.map_err(|e| {
            let error = e.into();
            anyhow::Error::new(DictatorError::PerformanceWarning {
                context: context.to_string(),
                message: error.to_string(),
                suggestion: suggestion.to_string(),
            })
        })
    }
}

/// Helper functions for common error scenarios
pub mod suggestions {

    /// Suggestions for configuration errors
    #[must_use]
    pub fn config_suggestions(error_type: &str) -> &'static str {
        match error_type {
            "invalid_toml" => {
                "Check TOML syntax using a validator. \
                 Ensure all brackets and quotes are properly closed."
            }
            "invalid_value" => {
                "Check the documentation for valid configuration values. \
                 Use 'dictator config validate' to verify your configuration."
            }
            "missing_field" => {
                "Add the required field to your configuration. \
                 Run 'dictator occupy' to generate a default configuration."
            }
            "file_not_found" => {
                "Ensure the configuration file exists and is readable. \
                 The default location is .dictate.toml in your project root."
            }
            _ => {
                "Check the error message and refer to the documentation \
                 for proper configuration syntax."
            }
        }
    }

    /// Suggestions for WASM loading errors
    #[must_use]
    pub fn wasm_suggestions(error_type: &str) -> &'static str {
        match error_type {
            "abi_mismatch" => {
                "Rebuild your WASM decree with the same dictator-decree-abi version \
                 as the host. Check 'dictator --version' for ABI compatibility."
            }
            "invalid_wasm" => {
                "Ensure your decree is compiled as a WASM component with the correct \
                 target (wasm32-wasip1). Check the build documentation."
            }
            "file_not_found" => {
                "Verify the WASM file path is correct and the file exists. \
                 Use absolute paths if relative paths are ambiguous."
            }
            "permission_denied" => {
                "Check file permissions and ensure the WASM file is readable \
                 by the current user."
            }
            _ => {
                "Check that your WASM decree is properly built and compatible \
                 with the current Dictator version."
            }
        }
    }

    /// Suggestions for file processing errors
    #[must_use]
    pub fn file_suggestions(error_type: &str) -> &'static str {
        match error_type {
            "read_error" => {
                "Check file permissions and ensure the file exists. \
                 Files may be locked by other processes."
            }
            "encoding_error" => {
                "Ensure files are UTF-8 encoded. \
                 Convert files from other encodings using iconv or similar tools."
            }
            "large_file" => {
                "Consider splitting large files into smaller modules \
                 or increasing max_lines in your configuration."
            }
            _ => {
                "Check file permissions, encoding, and ensure files \
                 are not corrupted or locked."
            }
        }
    }

    /// Suggestions for performance optimization
    #[must_use]
    pub fn performance_suggestions(issue: &str) -> &'static str {
        match issue {
            "many_files" => {
                "Consider using file patterns to limit the scope or run Dictator \
                 on specific directories instead of the entire project."
            }
            "large_files" => {
                "Split large files into smaller modules or increase memory limits. \
                 Consider using streaming mode for very large files."
            }
            "slow_wasm" => {
                "Check if WASM decrees are causing performance issues. \
                 Consider using native decrees or optimizing your WASM components."
            }
            "cache_miss" => {
                "Ensure WASM caching is enabled and working. \
                 Check cache statistics with appropriate verbosity flags."
            }
            _ => {
                "Monitor system resources and consider optimizing \
                 your project structure or Dictator configuration."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_config_error_display() {
        let error = DictatorError::ConfigError {
            file: PathBuf::from(".dictate.toml"),
            line: Some(42),
            message: "Invalid TOML syntax".to_string(),
            suggestion: "Check your brackets".to_string(),
        };

        let output = format!("{error}");
        assert!(output.contains("Configuration error"));
        assert!(output.contains(".dictate.toml:42"));
        assert!(output.contains("Invalid TOML syntax"));
        assert!(output.contains("💡 Suggestion: Check your brackets"));
    }

    #[test]
    fn test_context_extension() {
        use std::fs;

        let result: Result<String, std::io::Error> = fs::read_to_string("/nonexistent/file.toml");
        let enhanced_result = result.config_context(
            PathBuf::from(".dictate.toml"),
            Some(1),
            "Ensure the file exists and is readable",
        );

        assert!(enhanced_result.is_err());
        let error_msg = enhanced_result.unwrap_err().to_string();
        assert!(error_msg.contains("Configuration error"));
        assert!(error_msg.contains("💡 Suggestion:"));
    }
}
