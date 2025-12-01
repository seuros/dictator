// Rust file with mixed line endings and no final newline
pub trait Handler {
    fn handle(&self, event: String);
}

pub struct EventDispatcher {
    handlers: Vec<Box<dyn Handler>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        EventDispatcher {
            handlers: Vec::new(),
        }
    }

    pub fn register(&mut self, handler: Box<dyn Handler>) {
        self.handlers.push(handler);
    }

    pub fn dispatch(&self, event: String) {
        for handler in &self.handlers {
            handler.handle(event.clone());
        }
    }
}

pub struct LogHandler;

impl Handler for LogHandler {
    fn handle(&self, event: String) {
        println!("Event: {}", event);
    }
}

fn main() {
    let mut dispatcher = EventDispatcher::new();
    dispatcher.register(Box::new(LogHandler));
    dispatcher.dispatch("test".to_string());
}