use dapi_grpc::platform::v0::{GetIdentityRequest, GetIdentityResponse};
use dpp::prelude::Identity;
use proof::FromProof;
use prost::Message;

pub mod bindings;
pub mod proof;

/// Errors
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum Error {
    /// Drive error
    #[error("dash drive: {error}")]
    DriveError { error: String },

    /// Dash Protocol error
    #[error("dash protocol: {error}")]
    ProtocolError { error: String },

    /// Empty response
    #[error("empty response")]
    EmptyResponse,

    /// No proof in response
    #[error("no proof in response")]
    NoProof,

    /// Document not in proof
    #[error("requested document missing in proof")]
    DocumentMissingInProof,

    /// Decode protobuf error
    #[error("decode request protobuf: {error}")]
    ProtoRequestDecodeError { error: String },

    /// Decode protobuf response error
    #[error("decode response protobuf: {error}")]
    ProtoResponseDecodeError { error: String },

    /// Encode protobuf error
    #[error("encode protobuf: {error}")]
    ProtoEncodeError { error: String },
}

uniffi::include_scaffolding!("dash_drive_v0");
