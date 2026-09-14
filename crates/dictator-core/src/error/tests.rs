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
