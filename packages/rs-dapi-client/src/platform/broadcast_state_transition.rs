//! `BroadcastStateTransition` request.

use dapi_grpc::platform::v0::{self as platform_proto};
use drive::dpp::state_transition::StateTransition;

use super::IncompleteMessage;
use crate::{transport::TransportRequest, DapiRequest, Settings};

/// DAPI request to suggest a [StateTransition] to the network.
#[derive(Debug)]
pub struct BroadcastStateTransitionRequest {
    /// State transition to broadcast
    pub state_transition: StateTransition,
}

/// Response for [BroadcastStateTransition].
/// Cannot carry any info as this process is not synchronous to see any
/// results immediately except [StateTransition] was successfully broadcasted.
/// For actual results [WaitForStateTransitionResult](crate::platform::WaitForStateTransitionResult) is used.
#[derive(Debug)]
pub struct BroadcastStateTransitionResponse {}

impl DapiRequest for BroadcastStateTransitionRequest {
    type DapiResponse = BroadcastStateTransitionResponse;

    const SETTINGS_OVERRIDES: Settings = Settings::default();

    type Error = IncompleteMessage;

    type TransportRequest = platform_proto::BroadcastStateTransitionRequest;

    fn to_transport_request(&self) -> Self::TransportRequest {
        let mut cbor_state_transition = Vec::new();
        ciborium::into_writer(&self.state_transition, &mut cbor_state_transition)
            .expect("byte slice is a safe writer");
        platform_proto::BroadcastStateTransitionRequest {
            state_transition: cbor_state_transition,
        }
    }

    fn try_from_transport_response(
        _transport_response: <Self::TransportRequest as TransportRequest>::Response,
    ) -> Result<Self::DapiResponse, Self::Error> {
        Ok(BroadcastStateTransitionResponse {})
    }
}
