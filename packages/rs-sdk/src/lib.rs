//! Dash Platform Rust Software Development Kit
//!
//! This crate simplifies integration with Dash Platform from Rust.
//!
//! It includes methods to:
//! * prepare and execute queries against Dash Platform document repositories - in [query] module,
//! * verify query, contract and
//!

pub use dapi_grpc;

pub mod client;
pub mod query;
pub mod verify;

pub use drive::error::Error;

