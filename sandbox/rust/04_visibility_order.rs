// Rust file with wrong visibility ordering - pub after private
pub struct User {
    id: u32,
    pub name: String,
    email: String,
    pub age: u8,
    created_at: String,
    pub updated_at: String,
    internal_state: bool,
    pub metadata: String,
}

impl User {
    fn private_init() -> Self {
        User {
            id: 0,
            name: String::new(),
            email: String::new(),
            age: 0,
            created_at: String::new(),
            updated_at: String::new(),
            internal_state: false,
            metadata: String::new(),
        }
    }

    pub fn new(name: String, email: String, age: u8) -> Self {
        let mut user = Self::private_init();
        user.name = name;
        user.email = email;
        user.age = age;
        user
    }

    fn validate_email(email: &str) -> bool {
        email.contains('@')
    }

    pub fn set_email(&mut self, email: String) -> bool {
        if Self::validate_email(&email) {
            self.email = email;
            true
        } else {
            false
        }
    }

    fn get_id(&self) -> u32 {
        self.id
    }

    pub fn display_info(&self) {
        println!("User: {}, Age: {}", self.name, self.age);
    }
}

pub enum Role {
    Admin,
    User,
    Guest,
}

pub struct Permission {
    role: Role,
    pub can_edit: bool,
    read_only: bool,
    pub can_delete: bool,
}

impl Permission {
    fn new(role: Role) -> Self {
        Permission {
            role,
            can_edit: false,
            read_only: true,
            can_delete: false,
        }
    }

    pub fn for_admin() -> Self {
        let mut perm = Self::new(Role::Admin);
        perm.can_edit = true;
        perm.can_delete = true;
        perm.read_only = false;
        perm
    }

    pub fn display(&self) {
        println!("Edit: {}, Delete: {}", self.can_edit, self.can_delete);
    }
}

fn main() {
    let user = User::new("Alice".to_string(), "alice@example.com".to_string(), 30);
    user.display_info();

    let admin_perm = Permission::for_admin();
    admin_perm.display();
}
