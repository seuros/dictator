use super::*;

#[test]
fn native_decree_enabled_defaults_to_true() {
    let settings = dictator_core::config::DecreeSettings::default();

    assert!(native_decree_enabled(&settings));
}

#[test]
fn native_decree_enabled_respects_false() {
    let settings = dictator_core::config::DecreeSettings {
        enabled: Some(false),
        ..Default::default()
    };

    assert!(!native_decree_enabled(&settings));
}
