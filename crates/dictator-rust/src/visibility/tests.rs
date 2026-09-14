use crate::lint_source;

#[test]
fn detects_pub_after_private_in_struct() {
    let src = r"
struct User {
    name: String,
    age: u32,
    pub email: String,
}
";
    let diags = lint_source(src);
    assert!(
        diags.iter().any(|d| d.rule == "rust/visibility-order"),
        "Should detect pub field after private fields in struct"
    );
}

#[test]
fn detects_pub_after_private_in_impl() {
    let src = r"
impl User {
    fn private_method(&self) {}
    pub fn public_method(&self) {}
}
";
    let diags = lint_source(src);
    assert!(
        diags.iter().any(|d| d.rule == "rust/visibility-order"),
        "Should detect pub method after private method in impl"
    );
}

#[test]
fn accepts_pub_before_private() {
    let src = r"
struct User {
    pub id: u32,
    pub name: String,
    email: String,
}
";
    let diags = lint_source(src);
    assert!(
        !diags.iter().any(|d| d.rule == "rust/visibility-order"),
        "Should accept public fields before private fields"
    );
}

#[test]
fn accepts_impl_with_correct_order() {
    let src = r"
impl User {
    pub fn new(name: String) -> Self {
        User { name }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    fn validate(&self) -> bool {
        true
    }
}
";
    let diags = lint_source(src);
    assert!(
        !diags.iter().any(|d| d.rule == "rust/visibility-order"),
        "Should accept impl with public methods before private"
    );
}
