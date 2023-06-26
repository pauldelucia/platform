mod error;

/// The handlers of abci messages
pub mod handlers;

// new code - config,
#[cfg(feature = "server")]
pub mod config;
#[cfg(any(feature = "server", test))]
pub(crate) mod server;

pub use error::AbciError;
#[cfg(feature = "server")]
pub use server::start;
pub use server::AbciApplication;

/// Version of Drive sent to Tenderdash
pub const APP_VERSION: u64 = 1;
