use std::{future::Future, ops::DerefMut, sync::Arc};

use dapi_grpc::platform::v0::Proof;
use drive::dpp::prelude::{Identifier, Identity};
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tonic::codegen::{Body, Bytes, StdError};
use url::Url;

/// Data structures used in platform GRPC
pub use dapi_grpc::platform::v0 as platform;
pub use drive::query::DriveQuery;

use crate::error::Error;
use crate::proof::FromProof;
use crate::types::DapiResponse;
use dapi_grpc::platform::v0::platform_client::PlatformClient as GrpcPlatformClient;

/// configure and construct DAPI GRPC client
///
///
#[async_trait::async_trait]
pub trait PlatformClientBuilder<'a, T>
where
    T: 'a,
{
    fn new() -> Self;
    /// Set service URL (defaults to testnet)
    fn with_url(self, address: Url) -> Self;
    /// Configure for testnet; implies [with_url()]
    fn with_testnet() -> Self;
    /// Configure for production
    fn with_production() -> Self;
    /// Use provided Platform GRPC client
    fn with_grpc_client(self, client: GrpcPlatformClient<T>) -> Self;
    /// Allow insecure TLS connections, eg. with invalid and/or self-signed certificates
    fn with_insecure_tls(self, allow: bool) -> Self;

    /// Request proofs for each requst, and verify them every time
    fn with_proofs(enable: bool) -> Self;
    /// Build platform client working in sync mode
    fn build_sync(self) -> Result<PlatformClientSync<T>, Error>;
    async fn build_async(self) -> Result<PlatformClientAsync<T>, Error>;
}

pub struct PlatformClientAsync<T> {
    inner: GrpcPlatformClient<T>,
}

/// Synchronous client of Dash Platform
///
/// Use [`PlatformClientBuilder`] to instantiate.
///
/// Internally, it wraps [GrpcPlatformClient] in blocking calls.
pub struct PlatformClientSync<T> {
    inner: Arc<Mutex<GrpcPlatformClient<T>>>,
    // inner: Arc<GrpcPlatformClient<T>>,
    /// whether or not we use proofs whenever possible
    prove: bool,
}

impl<T> PlatformClientSync<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody> + Send + Sync + 'static,
    T::Future: Send,
    T::Error: Into<StdError> + Send,
    T::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    /// Call an async runtime in a blocking manner
    // fn call_blocking2<O, F: Fn() -> O>(&self, future: F) -> Result<O, Error>
    // where
    //     F: Send,
    //     O: Send,
    // {
    //     let (tx, rx) = oneshot::channel();

    //     tokio::spawn(async move {
    //         let result = future();
    //         tx.send(result);
    //     });

    //     Ok(rx
    //         .blocking_recv()
    //         .map_err(|e| Error::RuntimeError(e.to_string()))?)
    // }

    /// Call an async runtime in a blocking manner
    fn call_blocking<F: Future>(future: F) -> Result<F::Output, Error>
    where
        F: Send + 'static,
        F::Output: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();

        tokio::spawn(async move {
            let result = future.await;
            tx.send(result);
        });

        Ok(rx
            .blocking_recv()
            .map_err(|e| Error::RuntimeError(e.to_string()))?)
    }

    pub fn broadcast_state_transition(
        &mut self,
        request: platform::BroadcastStateTransitionRequest,
    ) -> std::result::Result<platform::BroadcastStateTransitionResponse, Error> {
        todo!()
    }

    pub fn get_identity(
        &mut self,
        identity_id: Identifier,
    ) -> std::result::Result<DapiResponse<Identity>, Error> {
        let request = platform::GetIdentityRequest {
            id: identity_id.to_vec(),
            prove: self.prove,
        };

        let grpc = Arc::clone(&self.inner);
        let request_fut = request.clone();
        let fut = async move {
            let mut guard = grpc.lock().await;
            let grpc = guard.deref_mut();

            let result = grpc.get_identity(request_fut).await;
            result
        };

        let response = Self::call_blocking(fut)??;
        let response = response.get_ref();

        let ret = match response.result.as_ref().ok_or(Error::EmptyResponse)? {
            platform::get_identity_response::Result::Identity(i) => {
                let identity = Identity::from_cbor(&i)?;
                DapiResponse {
                    response: identity,
                    proof: None,
                }
            }
            platform::get_identity_response::Result::Proof(proof) => {
                let identity = Identity::from_proof(&request, &response)?;

                DapiResponse {
                    response: identity,
                    proof: Some(proof.clone()),
                }
            }
        };

        Ok(ret)
    }

    pub fn get_identities(
        &mut self,
        request: platform::GetIdentitiesRequest,
    ) -> std::result::Result<platform::GetIdentitiesResponse, Error> {
        todo!()
    }

    pub fn get_identity_keys(
        &mut self,
        request: platform::GetIdentityKeysRequest,
    ) -> std::result::Result<platform::GetIdentityKeysResponse, Error> {
        todo!()
    }

    pub fn get_identity_balance(
        &mut self,
        request: platform::GetIdentityRequest,
    ) -> std::result::Result<platform::GetIdentityBalanceResponse, Error> {
        todo!()
    }

    pub fn get_identity_balance_and_revision(
        &mut self,
        request: platform::GetIdentityRequest,
    ) -> std::result::Result<platform::GetIdentityBalanceAndRevisionResponse, Error> {
        todo!()
    }

    pub fn get_proofs(
        &mut self,
        request: platform::GetProofsRequest,
    ) -> std::result::Result<platform::GetProofsResponse, Error> {
        todo!()
    }

    pub fn get_data_contract(
        &mut self,
        request: platform::GetDataContractRequest,
    ) -> std::result::Result<platform::GetDataContractResponse, Error> {
        todo!()
    }

    pub fn get_data_contracts(
        &mut self,
        request: platform::GetDataContractsRequest,
    ) -> std::result::Result<platform::GetDataContractsResponse, Error> {
        todo!()
    }

    pub fn get_documents(
        &mut self,
        request: DriveQuery,
    ) -> std::result::Result<platform::GetDocumentsResponse, Error> {
        todo!()
    }

    pub fn get_identities_by_public_key_hashes(
        &mut self,
        request: platform::GetIdentitiesByPublicKeyHashesRequest,
    ) -> std::result::Result<platform::GetIdentitiesByPublicKeyHashesResponse, Error> {
        todo!()
    }

    pub fn get_identity_by_public_key_hashes(
        &mut self,
        request: platform::GetIdentityByPublicKeyHashesRequest,
    ) -> std::result::Result<platform::GetIdentityByPublicKeyHashesResponse, Error> {
        todo!()
    }

    pub fn wait_for_state_transition_result(
        &mut self,
        request: platform::WaitForStateTransitionResultRequest,
    ) -> std::result::Result<platform::WaitForStateTransitionResultResponse, Error> {
        todo!()
    }

    pub fn get_consensus_params(
        &mut self,
        request: platform::GetConsensusParamsRequest,
    ) -> std::result::Result<platform::GetConsensusParamsResponse, Error> {
        todo!()
    }
}
