use super::*;

#[test]
fn test_default_config_is_valid_toml() {
    // Ensure the default config parses as valid TOML
    let parsed: Result<toml::Value, _> = toml::from_str(DEFAULT_CONFIG);
    assert!(parsed.is_ok(), "Default config must be valid TOML: {:?}", parsed.err());
}

#[test]
fn test_default_config_has_supreme_decree() {
    let config: toml::Value = toml::from_str(DEFAULT_CONFIG).unwrap();
    assert!(
        config.get("decree").and_then(|d| d.get("supreme")).is_some(),
        "Default config must include decree.supreme"
    );
}

#[test]
fn test_default_config_structure() {
    // Validate that config can be deserialized into DictateConfig
    use dictator_core::DictateConfig;
    let config: Result<DictateConfig, _> = toml::from_str(DEFAULT_CONFIG);
    assert!(
        config.is_ok(),
        "Default config must match DictateConfig structure: {:?}",
        config.err()
    );
}
