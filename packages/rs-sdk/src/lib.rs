//! Dash Platform Rust Software Development Kit
//!
//! This crate simplifies integration with Dash Platform from Rust.
//!
//! It includes methods to:
//! * prepare and execute queries against Dash Platform document repositories - in [query] module,
//! * verify query, contract and
//!

/// GPRC client for the Dash Platform, including data structures
pub mod client;
/// Errors returned by the SDK
pub mod error;
/// Proof verification functions
pub mod verify;
