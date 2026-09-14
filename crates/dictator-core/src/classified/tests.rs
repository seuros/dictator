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
        assert!(
            is_classified(Utf8Path::new(secret)),
            "{secret} should be classified"
        );
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
        assert!(
            !is_classified(Utf8Path::new(public)),
            "{public} should be lintable"
        );
    }
}
