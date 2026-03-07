//! Individual tool handlers for stalint, dictator, and watch commands.

mod dictator;
mod occupy;
mod stalint;
mod watch;

pub use dictator::handle_dictator;
pub use occupy::handle_occupy;
pub use stalint::handle_stalint;
pub use watch::{handle_stalint_unwatch, handle_stalint_watch};
