use url::Url;

use crate::dapi_grpc::platform::v0 as proto;
pub use crate::dapi_grpc::platform::v0::platform_client::PlatformClient as PlatformClientAsync;

#[async_trait::async_trait]
pub trait PlatformClientBuilder<T> {
    fn new() -> Self;
    fn with_address(self, address: Url) -> Self;
    fn with_client(self, client: PlatformClientAsync<T>) -> Self;
    /// Build platform client working in sync mode
    fn build_sync(self) -> Result<Box<dyn PlatformClientSync>, tonic::transport::Error>;
    async fn build_async(self) -> Result<PlatformClientAsync<T>, tonic::transport::Error>;
}

/// PlatformClientSync represents a synchronous client of Dash Platform.
///
/// Internally, it wraps [PlatformClientAsync] in blocking calls to Tokio runtime.
pub trait PlatformClientSync {
    fn broadcast_state_transition(
        &mut self,
        request: proto::BroadcastStateTransitionRequest,
    ) -> std::result::Result<proto::BroadcastStateTransitionResponse, tonic::Status>;

    fn get_identity(
        &mut self,
        request: proto::GetIdentityRequest,
    ) -> std::result::Result<proto::GetIdentityResponse, tonic::Status>;

    fn get_identities(
        &mut self,
        request: proto::GetIdentitiesRequest,
    ) -> std::result::Result<proto::GetIdentitiesResponse, tonic::Status>;

    fn get_identity_keys(
        &mut self,
        request: proto::GetIdentityKeysRequest,
    ) -> std::result::Result<proto::GetIdentityKeysResponse, tonic::Status>;

    fn get_identity_balance(
        &mut self,
        request: proto::GetIdentityRequest,
    ) -> std::result::Result<proto::GetIdentityBalanceResponse, tonic::Status>;

    fn get_identity_balance_and_revision(
        &mut self,
        request: proto::GetIdentityRequest,
    ) -> std::result::Result<proto::GetIdentityBalanceAndRevisionResponse, tonic::Status>;

    fn get_proofs(
        &mut self,
        request: proto::GetProofsRequest,
    ) -> std::result::Result<proto::GetProofsResponse, tonic::Status>;

    fn get_data_contract(
        &mut self,
        request: proto::GetDataContractRequest,
    ) -> std::result::Result<proto::GetDataContractResponse, tonic::Status>;

    fn get_data_contracts(
        &mut self,
        request: proto::GetDataContractsRequest,
    ) -> std::result::Result<proto::GetDataContractsResponse, tonic::Status>;

    fn get_documents(
        &mut self,
        request: proto::GetDocumentsRequest,
    ) -> std::result::Result<proto::GetDocumentsResponse, tonic::Status>;

    fn get_identities_by_public_key_hashes(
        &mut self,
        request: proto::GetIdentitiesByPublicKeyHashesRequest,
    ) -> std::result::Result<proto::GetIdentitiesByPublicKeyHashesResponse, tonic::Status>;

    fn get_identity_by_public_key_hashes(
        &mut self,
        request: proto::GetIdentityByPublicKeyHashesRequest,
    ) -> std::result::Result<proto::GetIdentityByPublicKeyHashesResponse, tonic::Status>;

    fn wait_for_state_transition_result(
        &mut self,
        request: proto::WaitForStateTransitionResultRequest,
    ) -> std::result::Result<proto::WaitForStateTransitionResultResponse, tonic::Status>;

    fn get_consensus_params(
        &mut self,
        request: proto::GetConsensusParamsRequest,
    ) -> std::result::Result<proto::GetConsensusParamsResponse, tonic::Status>;
}

struct Client<T> {
    inner: PlatformClientAsync<T>,
}

impl<T> Client<T> {}
