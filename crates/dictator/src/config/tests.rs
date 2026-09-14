use super::*;

fn write_config(contents: &[u8]) -> (tempfile::NamedTempFile, Utf8PathBuf) {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut file, contents).unwrap();
    let path = Utf8PathBuf::from_path_buf(file.path().to_path_buf()).unwrap();
    (file, path)
}

#[test]
fn load_dictate_config_applies_active_profile() {
    let (_file, path) = write_config(
        br#"
active_profile = "ci"

[decree.supreme]
max_line_length = 120

[profile.ci.decree.supreme]
max_line_length = 80
"#,
    );

    let config = load_dictate_config(Some(&path), None).unwrap().unwrap();

    assert_eq!(config.decree["supreme"].max_line_length, Some(80));
}

#[test]
fn load_dictate_config_explicit_profile_overrides_active_profile() {
    let (_file, path) = write_config(
        br#"
active_profile = "ci"

[decree.supreme]
max_line_length = 120

[profile.ci.decree.supreme]
max_line_length = 80

[profile.relaxed.decree.supreme]
max_line_length = 140
"#,
    );

    let config = load_dictate_config(Some(&path), Some("relaxed"))
        .unwrap()
        .unwrap();

    assert_eq!(config.decree["supreme"].max_line_length, Some(140));
}

#[test]
fn load_dictate_config_applies_default_profile_when_present() {
    let (_file, path) = write_config(
        br#"
[decree.supreme]
max_line_length = 120

[profile.default.decree.supreme]
max_line_length = 100
"#,
    );

    let config = load_dictate_config(Some(&path), None).unwrap().unwrap();

    assert_eq!(config.decree["supreme"].max_line_length, Some(100));
}
