//! Dictator library interface for integration tests

#![allow(
    clippy::needless_pass_by_value,
    clippy::similar_names,
    clippy::too_many_lines
)]

// Only expose modules needed for integration testing
pub mod cli;
pub mod occupy;
