use dpp::consensus::codes::ErrorWithCode as DppErrorWithCode;

use super::Error;

/// Trait for errors that have code
pub trait ErrorWithCode {
    /// Get error code
    fn code(&self) -> u32;
}

impl ErrorWithCode for Error {
    fn code(&self) -> u32 {
        match self {
            // TODO: figure out what else should have own code here
            Self::Protocol(drive::dpp::errors::ProtocolError::ConsensusError(ce)) => ce.code(),
            _ => GrpcErrorCodes::INTERNAL as u32,
        }
    }
}

/// Enum describing all the possible Grpc error codes
#[repr(u8)]
pub enum GrpcErrorCodes {
    /// Canceled
    CANCELLED = 1,
    /// Unknown
    UNKNOWN = 2,
    /// Invalid argument
    INVALID_ARGUMENT = 3,
    /// Deadline exceeded
    DEADLINE_EXCEEDED = 4,
    /// Not found
    NOT_FOUND = 5,
    /// Already exists
    ALREADY_EXISTS = 6,
    /// Permission denied
    PERMISSION_DENIED = 7,
    /// Resource exhausted
    RESOURCE_EXHAUSTED = 8,
    /// Failed precondition
    FAILED_PRECONDITION = 9,
    /// Aborted
    ABORTED = 10,
    /// Out of range
    OUT_OF_RANGE = 11,
    /// Unimplemented
    UNIMPLEMENTED = 12,
    /// Internal
    INTERNAL = 13,
    /// Unavailable
    UNAVAILABLE = 14,
    /// Data loss
    DATA_LOSS = 15,
    /// Unauthenticated
    UNAUTHENTICATED = 16,
    /// Version mismatch
    VERSION_MISMATCH = 100,
}
