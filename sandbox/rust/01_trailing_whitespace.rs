// Rust file with trailing whitespace violations   
pub struct Config {
    pub name: String,   
    pub version: String,	
}

impl Config {
    pub fn new(name: String, version: String) -> Self {
        Config { name, version }   
    }

    pub fn display(&self) {
        println!("Name: {}", self.name);
        println!("Version: {}", self.version);  	
    }
}

fn main() {
    let config = Config::new(
        "MyApp".to_string(),   
        "1.0.0".to_string(),
    );
    config.display();
}   
