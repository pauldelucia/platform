use url::Url;

/// Data structures used in platform GRPC
pub use dapi_grpc::platform::v0 as platform;
pub use drive::query::DriveQuery;

use crate::error::Error;
use dapi_grpc::platform::v0::platform_client::PlatformClient as GrpcPlatformClient;

/// configure and construct DAPI GRPC client
///
///
#[async_trait::async_trait]
pub trait PlatformClientBuilder<T> {
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
    /// Build platform client working in sync mode
    fn build_sync(self) -> Result<PlatformClientSync<T>, Error>;
    async fn build_async(self) -> Result<PlatformClientAsync<T>, Error>;
}

pub struct PlatformClientAsync<T> {
    inner: GrpcPlatformClient<T>,
}

/// Synchronous client of Dash Platform
///
/// Internally, it wraps [GrpcPlatformClient] in blocking calls.
pub struct PlatformClientSync<T> {
    inner: GrpcPlatformClient<T>,
}

impl<T> PlatformClientSync<T> {
    pub fn broadcast_state_transition(
        &mut self,
        request: platform::BroadcastStateTransitionRequest,
    ) -> std::result::Result<platform::BroadcastStateTransitionResponse, Error> {
        todo!()
    }

    pub fn get_identity(
        &mut self,
        request: platform::GetIdentityRequest,
    ) -> std::result::Result<platform::GetIdentityResponse, Error> {
        todo!()
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
