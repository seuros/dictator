// Rust file with mixed tabs and spaces
pub enum Status {
	Active,
    Inactive,
	Pending,
}

pub struct Task {
	pub id: u32,
    pub name: String,
	pub status: Status,
}

impl Task {
	pub fn new(id: u32, name: String) -> Self {
		Task {
			id,
			name,
			status: Status::Pending,
		}
	}

	pub fn is_active(&self) -> bool {
		matches!(self.status, Status::Active)
	}
}

fn main() {
	let task = Task::new(1, "Important".to_string());
	println!("Task active: {}", task.is_active());
}
