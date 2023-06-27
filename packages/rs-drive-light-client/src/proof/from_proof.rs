use std::fmt::Debug;

use dapi_grpc::platform::v0::{self as platform};
use dpp::{
    prelude::{Identifier, Identity},
};
pub use drive::drive::verify::RootHash;
use drive::drive::Drive;
use tenderdash_abci::{
    proto::{
        google::protobuf::Timestamp,
        serializers::timestamp::FromMilis,
        types::{CanonicalVote, SignedMsgType, StateId},
    },
    signatures::{SignBytes, SignDigest},
};

use crate::Error;

use super::verify::verify_tenderdash_proof;

// #[cfg(feature = "mockall")]

/// Create an object based on proof received from DAPI
///
/// # Arguments
///
/// * request: request sent to the server
/// * response: response received
///
/// # example
///
/// ```no_run
/// #  tokio::runtime::Runtime::new().unwrap().block_on(async {
/// use rs_sdk::proof::FromProof;
/// use dapi_grpc::platform::v0::{
///     platform_client::PlatformClient as GrpcPlatformClient, GetIdentityRequest,
/// };
/// use dpp::prelude::Identity;
///
/// let mut grpc = GrpcPlatformClient::connect("http://127.0.0.1:1234")
///     .await
///     .unwrap();
/// let request = GetIdentityRequest {
///     id: vec![0u8; 32],
///     prove: true,
/// };
/// let response = grpc.get_identity(request.clone()).await.unwrap();
/// let response = response.get_ref();
/// let identity = Identity::maybe_from_proof(&request, response);
/// assert!(identity.is_ok().is_some());
/// # });
/// ```
pub trait FromProof<Req, Resp> {
    fn maybe_from_proof(
        request: &Req,
        response: &Resp,
        provider: Box<dyn QuorumInfoProvider>,
    ) -> Result<Option<Self>, Error>
    where
        Self: Sized;

    fn from_proof(
        request: &Req,
        response: &Resp,
        provider: Box<dyn QuorumInfoProvider>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Self::maybe_from_proof(request, response, provider)?.ok_or(Error::DocumentMissingInProof)
    }
}

#[uniffi::export(callback_interface)]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait QuorumInfoProvider: Send + Sync {
    fn get_quorum_public_key(&self, quorum_hash: Vec<u8>) -> Result<Vec<u8>, Error>;
}

#[cfg_attr(feature = "mock", mockall::automock)]
impl FromProof<platform::GetIdentityRequest, platform::GetIdentityResponse> for Identity {
    fn maybe_from_proof(
        request: &platform::GetIdentityRequest,
        response: &platform::GetIdentityResponse,
        provider: Box<dyn QuorumInfoProvider>,
    ) -> Result<Option<Self>, Error> {
        // Parse response to read proof and metadata
        let proof = match response.result.as_ref().ok_or(Error::EmptyResponse)? {
            platform::get_identity_response::Result::Proof(p) => p,
            platform::get_identity_response::Result::Identity(_) => {
                return Err(Error::EmptyResponseProof)
            }
        };

        let mtd = response
            .metadata
            .as_ref()
            .ok_or(Error::EmptyResponseMetadata)?;

        // Load some info from request
        let id = Identifier::from_bytes(&request.id).map_err(|e| Error::ProtocolError {
            error: e.to_string(),
        })?;

        // Extract content from proof and verify Drive/GroveDB proofs
        let (root_hash, maybe_identity) = Drive::verify_full_identity_by_identity_id(
            &proof.grovedb_proof,
            false,
            id.into_buffer(),
        )
        .map_err(|e| Error::DriveError {
            error: e.to_string(),
        })?;

        verify_tenderdash_proof(proof, mtd, &root_hash, provider)?;

        Ok(maybe_identity)
    }
}
