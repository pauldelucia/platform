use std::fmt::Debug;

use dapi_grpc::{
    core::v0::{core_client, get_block_request::Block},
    platform::v0::{self as platform, Proof, ResponseMetadata},
};
use dpp::{
    bls_signatures,
    prelude::{Identifier, Identity},
    ProtocolError,
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
pub trait QuorumInfoProvider: Send + Sync {
    fn get_quorum_type(&self, height: u64, quorum_hash: Vec<u8>) -> Result<u8, Error>;
    fn get_quorum_public_key(&self, height: u64, quorum_hash: Vec<u8>) -> Result<Vec<u8>, Error>;
}

impl<T: QuorumInfoProvider> QuorumInfoProvider for Box<T> {
    fn get_quorum_public_key(&self, height: u64, quorum_hash: Vec<u8>) -> Result<Vec<u8>, Error> {
        (**self).get_quorum_public_key(height, quorum_hash)
    }
    fn get_quorum_type(&self, height: u64, quorum_hash: Vec<u8>) -> Result<u8, Error> {
        (**self).get_quorum_type(height, quorum_hash)
    }
}
//  {
//     fn get_quorum_public_key(&self) {
//         self.as_ref().get_quorum_public_key()
//     }
//     fn get_quorum_type(&self) -> u8 {
//         self.as_ref().get_quorum_type()
//     }
// }
pub struct MockQuorumInfoProvider;

impl QuorumInfoProvider for MockQuorumInfoProvider {
    fn get_quorum_public_key(&self, height: u64, quorum_hash: Vec<u8>) -> Result<Vec<u8>, Error> {
        Ok(vec![0u8; 32])
    }
    fn get_quorum_type(&self, height: u64, quorum_hash: Vec<u8>) -> Result<u8, Error> {
        Ok(0)
    }
}

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

        // Verify Tenderdash proof

        //pub struct Proof {
        //     #[prost(bytes = "vec", tag = "1")]
        //     pub grovedb_proof: ::prost::alloc::vec::Vec<u8>,
        //     #[prost(bytes = "vec", tag = "2")]
        //     pub quorum_hash: ::prost::alloc::vec::Vec<u8>,
        //     #[prost(bytes = "vec", tag = "3")]
        //     pub signature: ::prost::alloc::vec::Vec<u8>,
        //     #[prost(uint32, tag = "4")]
        //     pub round: u32,
        // }

        Ok(maybe_identity)
    }
}

pub fn verify_tenderdash_proof<P: QuorumInfoProvider>(
    proof: Proof,
    mtd: ResponseMetadata,
    root_hash: &[u8],
    quorum_info_provider: P,
) -> Result<(), Error> {
    // TODO fill the following
    let chain_id = "abc";
    let block_id_hash: Vec<u8> = vec![0u8; 32];
    let quorum_type = 0;
    let pubkey_bytes = [0u8; 32];

    let version = mtd.protocol_version as u64;
    let height = mtd.height as u64;
    let round = proof.round as u32;

    let state_id = StateId {
        app_version: version,
        core_chain_locked_height: mtd.core_chain_locked_height,
        time: Some(Timestamp::from_milis(mtd.time_ms as i64)),
        app_hash: root_hash.into(),
        height,
    };

    let state_id_hash = state_id
        .sha256(chain_id, mtd.height as i64, proof.round as i32)
        .expect("failed to calculate state id hash");

    let commit = CanonicalVote {
        r#type: SignedMsgType::Precommit.into(),
        block_id: block_id_hash,
        chain_id: chain_id.to_string(),
        height: mtd.height as i64,
        round: proof.round as i64,
        state_id: state_id_hash,
    };

    // Now, lookup quorum details
    let quorum_hash =
        TryInto::<[u8; 32]>::try_into(proof.quorum_hash).map_err(|e| Error::InvalidQuorum {
            error: "invalid quorum hash size".to_string(),
        })?;

    // Verify signature
    let sign_digest = commit
        .sign_digest(
            chain_id,
            quorum_type,
            &quorum_hash,
            height as i64,
            round as i32,
        )
        .map_err(|e| Error::SignDigestFailed {
            error: e.to_string(),
        })?;

    let signature = TryInto::<[u8; 96]>::try_into(proof.signature).map_err(|e| {
        Error::InvalidSignatureFormat {
            error: "invalid signature size".to_string(),
        }
    })?;

    let pubkey = bls_signatures::PublicKey::from_bytes(&pubkey_bytes).map_err(|e| {
        Error::InvalidPublicKey {
            error: e.to_string(),
        }
    })?;

    match verify_signature_digest(&sign_digest, &signature, &pubkey)? {
        true => Ok(()),
        false => Err(Error::InvalidSignature {
            error: "signature mismatch".to_string(),
        }),
    }
}

/// Verify signature for sign_digest, using public_key
pub fn verify_signature_digest(
    sign_digest: &[u8],
    signature: &[u8; 96],
    public_key: &bls_signatures::PublicKey,
) -> Result<bool, Error> {
    if signature == &[0; 96] {
        return Err(Error::SignatureVerificationError {
            error: "empty signature".to_string(),
        });
    }
    let signature = bls_signatures::Signature::from_bytes(signature).map_err(|e| {
        Error::SignatureVerificationError {
            error: e.to_string(),
        }
    })?;

    Ok(public_key.verify(&signature, sign_digest))
}
