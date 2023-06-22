use dapi_grpc::platform::v0 as platform;
use dpp::{
    prelude::{Identifier, Identity},
    ProtocolError,
};
pub use drive::drive::verify::RootHash;
use drive::drive::Drive;

use crate::error::Error;

// #[cfg(feature = "mockall")]

pub mod data_contract;
pub mod document;
pub mod identity;

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
/// let identity = Identity::from_proof(&request, response);
/// assert!(identity.is_ok());
/// # });
/// ```
pub trait FromProof<Req, Resp> {
    fn from_proof(request: &Req, response: &Resp) -> Result<Self, Error>
    where
        Self: Sized;
}

impl FromProof<platform::GetIdentityRequest, platform::GetIdentityResponse> for Identity {
    fn from_proof(
        request: &platform::GetIdentityRequest,
        response: &platform::GetIdentityResponse,
    ) -> Result<Self, Error> {
        let (_, maybe_identity) = if let platform::get_identity_response::Result::Proof(p) =
            response.result.as_ref().ok_or(Error::NoProof)?
        {
            let id = Identifier::from_bytes(&request.id).map_err(|e| ProtocolError::from(e))?;
            Drive::verify_full_identity_by_identity_id(&p.grovedb_proof, false, id.into_buffer())?
        } else {
            return Err(Error::EmptyResponse);
        };

        maybe_identity.ok_or(Error::EmptyResponse)
    }
}
