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
}
