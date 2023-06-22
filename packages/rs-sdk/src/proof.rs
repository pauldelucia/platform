use dapi_grpc::platform::v0 as platform;
pub use drive::drive::verify::RootHash;
use drive::{
    dpp::{
        prelude::{Identifier, Identity},
        ProtocolError,
    },
    drive::Drive,
};

use crate::error::Error;

// #[cfg(feature = "mockall")]

pub mod data_contract;
pub mod document;
pub mod identity;

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
