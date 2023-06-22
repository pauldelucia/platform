use dpp::{prelude::Identifier, ProtocolError};

/// Errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid GRPC response
    #[error("grpc response status: {0}")]
    GrpcStatus(#[from] tonic::Status),

    /// Transport layer error
    #[error("grpc transport: {0}")]
    GrpcTransport(#[from] tonic::transport::Error),

    /// Drive error
    #[error("dash drive: {0}")]
    DriveError(#[from] drive::error::Error),

    /// Dash Protocol error
    #[error("dash protocol: {0}")]
    ProtocolError(#[from] ProtocolError),

    /// Empty response
    #[error("empty response")]
    EmptyResponse,

    /// No proof in response
    #[error("no proof in response")]
    NoProof,

    /// Document not in proof
    #[error("requested document missing in proof: {0}")]
    DocumentMissingInProof(Identifier),

    /// Async runtime error
    #[error("internal runtime error: {0}")]
    RuntimeError(String),
}
