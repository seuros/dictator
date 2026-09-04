//! Classified-material detection: files whose contents must never be inspected,
//! quoted in snippets, or auto-fixed.

use camino::Utf8Path;

/// Rule id emitted for classified files
pub const CLASSIFIED_RULE: &str = "supreme/classified";

const CLASSIFIED_EXTENSIONS: &[&str] =
    &["pem", "key", "enc", "p12", "pfx", "der", "jks", "keystore", "kdbx", "keytab"];

const CLASSIFIED_FILENAMES: &[&str] = &[
    "master.key",
    "credentials.yml.enc",
    "secrets.yml",
    "id_rsa",
    "id_ed25519",
    "id_ecdsa",
    "id_dsa",
    ".netrc",
    ".pgpass",
    ".npmrc",
];

// Committed placeholders are lintable
const TEMPLATE_SUFFIXES: &[&str] = &[".example", ".sample", ".template", ".dist"];

/// True when a file's contents are secret material
#[must_use]
pub fn is_classified(path: &Utf8Path) -> bool {
    let Some(name) = path.file_name() else {
        return false;
    };
    let lower = name.to_ascii_lowercase();

    if TEMPLATE_SUFFIXES.iter().any(|suffix| lower.ends_with(suffix)) {
        return false;
    }
    if CLASSIFIED_FILENAMES.contains(&lower.as_str()) || lower.starts_with(".env") {
        return true;
    }
    path.extension()
        .is_some_and(|ext| CLASSIFIED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
}

#[cfg(test)]
mod tests {
    use super::is_classified;
    use camino::Utf8Path;

    #[test]
    fn classification_table() {
        for secret in [
            "config/master.key",
            "config/credentials.yml.enc",
            "certs/server.pem",
            "tls/private.KEY",
            ".env",
            ".env.production",
            ".ssh/id_rsa",
            ".netrc",
            "store.p12",
        ] {
            assert!(is_classified(Utf8Path::new(secret)), "{secret} should be classified");
        }
        for public in [
            ".env.example",
            ".env.sample",
            "config.template",
            "src/main.rs",
            "README.md",
            ".ssh/id_rsa.pub",
            "keyboard.rs",
        ] {
            assert!(!is_classified(Utf8Path::new(public)), "{public} should be lintable");
        }
    }
}
