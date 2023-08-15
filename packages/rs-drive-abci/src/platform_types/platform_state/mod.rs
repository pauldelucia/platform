/// Version 0
pub mod v0;

use crate::error::Error;
use crate::platform_types::platform_state::v0::{
    PlatformInitializationState, PlatformStateForSavingV0, PlatformStateV0, PlatformStateV0Methods,
};

use crate::platform_types::validator_set::ValidatorSet;
use crate::rpc::core::QuorumListExtendedInfo;
use dashcore_rpc::dashcore_rpc_json::{MasternodeListItem, QuorumType};
use derive_more::From;
use dpp::bincode::{config, Decode, Encode};
use dpp::block::epoch::Epoch;
use dpp::block::extended_block_info::ExtendedBlockInfo;
use dpp::dashcore::{ProTxHash, QuorumHash};
use dpp::serialization::{
    PlatformDeserializable, PlatformDeserializableFromVersionedStructure, PlatformSerializable,
};
use dpp::util::deserializer::ProtocolVersion;

use dpp::version::{PlatformVersion, TryFromPlatformVersioned, TryIntoPlatformVersioned};
use dpp::ProtocolError;
use dpp::ProtocolError::{PlatformDeserializationError, PlatformSerializationError};
use indexmap::IndexMap;

use crate::error::execution::ExecutionError;
use std::collections::{BTreeMap, HashMap};

/// Platform state
#[derive(Clone, Debug, From)]
pub enum PlatformState {
    /// Version 0
    V0(PlatformStateV0),
}

/// Platform state
#[derive(Clone, Debug, Encode, Decode, From)]
enum PlatformStateForSaving {
    /// Version 0
    V0(PlatformStateForSavingV0),
}

impl PlatformStateForSaving {
    pub fn current_protocol_version_in_consensus(&self) -> ProtocolVersion {
        match self {
            PlatformStateForSaving::V0(v0) => v0.current_protocol_version_in_consensus,
        }
    }
}

impl PlatformSerializable for PlatformState {
    type Error = Error;

    fn serialize(&self) -> Result<Vec<u8>, Self::Error> {
        let platform_version = PlatformVersion::get(self.current_protocol_version_in_consensus())?;
        let config = config::standard().with_big_endian().with_no_limit();
        let platform_state_for_saving: PlatformStateForSaving =
            self.clone().try_into_platform_versioned(platform_version)?;
        bincode::encode_to_vec(platform_state_for_saving, config).map_err(|e| {
            PlatformSerializationError(format!("unable to serialize PlatformState: {}", e)).into()
        })
    }
}

// The version we should deserialize this into is determined by the actual saved state
impl PlatformDeserializable for PlatformState {
    fn deserialize_no_limit(data: &[u8]) -> Result<Self, ProtocolError>
    where
        Self: Sized,
    {
        let config = config::standard().with_big_endian().with_no_limit();
        let platform_state_in_save_format: PlatformStateForSaving =
            bincode::decode_from_slice(data, config)
                .map_err(|e| {
                    PlatformDeserializationError(format!(
                        "unable to deserialize PlatformStateForSaving: {}",
                        e
                    ))
                })?
                .0;
        let platform_version = PlatformVersion::get(
            platform_state_in_save_format.current_protocol_version_in_consensus(),
        )?;
        platform_state_in_save_format.try_into_platform_versioned(platform_version)
    }
}

impl PlatformDeserializableFromVersionedStructure for PlatformState {
    fn versioned_deserialize(
        data: &[u8],
        platform_version: &PlatformVersion,
    ) -> Result<Self, ProtocolError>
    where
        Self: Sized,
    {
        let config = config::standard().with_big_endian().with_no_limit();
        let platform_state_in_save_format: PlatformStateForSaving =
            bincode::decode_from_slice(data, config)
                .map_err(|e| {
                    PlatformDeserializationError(format!(
                        "unable to deserialize PlatformStateForSaving: {}",
                        e
                    ))
                })?
                .0;
        platform_state_in_save_format.try_into_platform_versioned(platform_version)
    }
}

impl PlatformState {
    /// Get the current platform version
    pub fn current_platform_version(&self) -> Result<&'static PlatformVersion, Error> {
        Ok(PlatformVersion::get(
            self.current_protocol_version_in_consensus(),
        )?)
    }

    /// The default state at platform start
    pub fn default_with_protocol_versions(
        current_protocol_version_in_consensus: ProtocolVersion,
        next_epoch_protocol_version: ProtocolVersion,
    ) -> PlatformState {
        //todo find the current Platform state for the protocol version
        PlatformStateV0::default_with_protocol_versions(
            current_protocol_version_in_consensus,
            next_epoch_protocol_version,
        )
        .into()
    }

    /// Retrieve version 0, or an error if not currently on version 0
    pub fn v0(&self) -> Result<&PlatformStateV0, Error> {
        match self {
            PlatformState::V0(v) => Ok(v),
        }
    }

    /// Retrieve version 0 as mutable, or an error if not currently on version 0
    pub fn v0_mut(&mut self) -> Result<&mut PlatformStateV0, Error> {
        match self {
            PlatformState::V0(v) => Ok(v),
        }
    }
}

impl TryFromPlatformVersioned<PlatformState> for PlatformStateForSaving {
    type Error = Error;
    fn try_from_platform_versioned(
        value: PlatformState,
        platform_version: &PlatformVersion,
    ) -> Result<Self, Self::Error> {
        match value {
            PlatformState::V0(v0) => {
                match platform_version
                    .drive_abci
                    .structs
                    .platform_state_for_saving_structure
                {
                    0 => {
                        let saving_v0: PlatformStateForSavingV0 = v0.try_into()?;
                        Ok(saving_v0.into())
                    }
                    version => Err(Error::Execution(ExecutionError::UnknownVersionMismatch {
                        method:
                            "PlatformStateForSaving::try_from_platform_versioned(PlatformState)"
                                .to_string(),
                        known_versions: vec![0],
                        received: version,
                    })),
                }
            }
        }
    }
}

impl TryFromPlatformVersioned<PlatformStateForSaving> for PlatformState {
    type Error = ProtocolError;

    fn try_from_platform_versioned(
        value: PlatformStateForSaving,
        platform_version: &PlatformVersion,
    ) -> Result<Self, Self::Error> {
        match value {
            PlatformStateForSaving::V0(v0) => {
                match platform_version.drive_abci.structs.platform_state_structure {
                    0 => {
                        let platform_state_v0: PlatformStateV0 = v0.into();
                        Ok(platform_state_v0.into())
                    }
                    version => Err(ProtocolError::UnknownVersionMismatch {
                        method:
                            "PlatformState::try_from_platform_versioned(PlatformStateForSaving)"
                                .to_string(),
                        known_versions: vec![0],
                        received: version,
                    }),
                }
            }
        }
    }
}

impl PlatformStateV0Methods for PlatformState {
    fn height(&self) -> u64 {
        match self {
            PlatformState::V0(v0) => v0.height(),
        }
    }

    fn known_height_or(&self, default: u64) -> u64 {
        match self {
            PlatformState::V0(v0) => v0.known_height_or(default),
        }
    }

    fn core_height(&self) -> u32 {
        match self {
            PlatformState::V0(v0) => v0.core_height(),
        }
    }

    fn known_core_height_or(&self, default: u32) -> u32 {
        match self {
            PlatformState::V0(v0) => v0.known_core_height_or(default),
        }
    }

    fn last_block_time_ms(&self) -> Option<u64> {
        match self {
            PlatformState::V0(v0) => v0.last_block_time_ms(),
        }
    }

    fn last_quorum_hash(&self) -> [u8; 32] {
        match self {
            PlatformState::V0(v0) => v0.last_quorum_hash(),
        }
    }

    fn last_block_signature(&self) -> [u8; 96] {
        match self {
            PlatformState::V0(v0) => v0.last_block_signature(),
        }
    }

    fn last_block_app_hash(&self) -> Option<[u8; 32]> {
        match self {
            PlatformState::V0(v0) => v0.last_block_app_hash(),
        }
    }

    fn last_block_height(&self) -> u64 {
        match self {
            PlatformState::V0(v0) => v0.last_block_height(),
        }
    }

    fn last_block_round(&self) -> u32 {
        match self {
            PlatformState::V0(v0) => v0.last_block_round(),
        }
    }

    fn epoch(&self) -> Epoch {
        match self {
            PlatformState::V0(v0) => v0.epoch(),
        }
    }

    fn epoch_ref(&self) -> &Epoch {
        match self {
            PlatformState::V0(v0) => v0.epoch_ref(),
        }
    }

    fn hpmn_list_len(&self) -> usize {
        match self {
            PlatformState::V0(v0) => v0.hpmn_list_len(),
        }
    }

    fn current_validator_set(&self) -> Result<&ValidatorSet, Error> {
        match self {
            PlatformState::V0(v0) => v0.current_validator_set(),
        }
    }

    fn current_protocol_version_in_consensus(&self) -> ProtocolVersion {
        match self {
            PlatformState::V0(v0) => v0.current_protocol_version_in_consensus(),
        }
    }

    fn last_committed_block_info(&self) -> &Option<ExtendedBlockInfo> {
        match self {
            PlatformState::V0(v0) => &v0.last_committed_block_info,
        }
    }

    fn next_epoch_protocol_version(&self) -> ProtocolVersion {
        match self {
            PlatformState::V0(v0) => v0.next_epoch_protocol_version,
        }
    }

    fn quorums_extended_info(&self) -> &HashMap<QuorumType, QuorumListExtendedInfo> {
        match self {
            PlatformState::V0(v0) => &v0.quorums_extended_info,
        }
    }

    fn current_validator_set_quorum_hash(&self) -> QuorumHash {
        match self {
            PlatformState::V0(v0) => v0.current_validator_set_quorum_hash,
        }
    }

    fn next_validator_set_quorum_hash(&self) -> &Option<QuorumHash> {
        match self {
            PlatformState::V0(v0) => &v0.next_validator_set_quorum_hash,
        }
    }

    fn validator_sets(&self) -> &IndexMap<QuorumHash, ValidatorSet> {
        match self {
            PlatformState::V0(v0) => &v0.validator_sets,
        }
    }

    fn full_masternode_list(&self) -> &BTreeMap<ProTxHash, MasternodeListItem> {
        match self {
            PlatformState::V0(v0) => &v0.full_masternode_list,
        }
    }

    fn hpmn_masternode_list(&self) -> &BTreeMap<ProTxHash, MasternodeListItem> {
        match self {
            PlatformState::V0(v0) => &v0.hpmn_masternode_list,
        }
    }

    fn initialization_information(&self) -> &Option<PlatformInitializationState> {
        match self {
            PlatformState::V0(v0) => &v0.initialization_information,
        }
    }

    fn set_last_committed_block_info(&mut self, info: Option<ExtendedBlockInfo>) {
        match self {
            PlatformState::V0(v0) => v0.set_last_committed_block_info(info),
        }
    }

    fn set_current_protocol_version_in_consensus(&mut self, version: ProtocolVersion) {
        match self {
            PlatformState::V0(v0) => v0.set_current_protocol_version_in_consensus(version),
        }
    }

    fn set_next_epoch_protocol_version(&mut self, version: ProtocolVersion) {
        match self {
            PlatformState::V0(v0) => v0.set_next_epoch_protocol_version(version),
        }
    }

    fn set_quorums_extended_info(&mut self, info: HashMap<QuorumType, QuorumListExtendedInfo>) {
        match self {
            PlatformState::V0(v0) => v0.set_quorums_extended_info(info),
        }
    }

    fn set_current_validator_set_quorum_hash(&mut self, hash: QuorumHash) {
        match self {
            PlatformState::V0(v0) => v0.set_current_validator_set_quorum_hash(hash),
        }
    }

    fn set_next_validator_set_quorum_hash(&mut self, hash: Option<QuorumHash>) {
        match self {
            PlatformState::V0(v0) => v0.set_next_validator_set_quorum_hash(hash),
        }
    }

    fn set_validator_sets(&mut self, sets: IndexMap<QuorumHash, ValidatorSet>) {
        match self {
            PlatformState::V0(v0) => v0.set_validator_sets(sets),
        }
    }

    fn set_full_masternode_list(&mut self, list: BTreeMap<ProTxHash, MasternodeListItem>) {
        match self {
            PlatformState::V0(v0) => v0.set_full_masternode_list(list),
        }
    }

    fn set_hpmn_masternode_list(&mut self, list: BTreeMap<ProTxHash, MasternodeListItem>) {
        match self {
            PlatformState::V0(v0) => v0.set_hpmn_masternode_list(list),
        }
    }

    fn set_initialization_information(&mut self, info: Option<PlatformInitializationState>) {
        match self {
            PlatformState::V0(v0) => v0.set_initialization_information(info),
        }
    }

    fn last_committed_block_info_mut(&mut self) -> &mut Option<ExtendedBlockInfo> {
        match self {
            PlatformState::V0(v0) => v0.last_committed_block_info_mut(),
        }
    }

    fn current_protocol_version_in_consensus_mut(&mut self) -> &mut ProtocolVersion {
        match self {
            PlatformState::V0(v0) => v0.current_protocol_version_in_consensus_mut(),
        }
    }

    fn next_epoch_protocol_version_mut(&mut self) -> &mut ProtocolVersion {
        match self {
            PlatformState::V0(v0) => v0.next_epoch_protocol_version_mut(),
        }
    }

    fn quorums_extended_info_mut(&mut self) -> &mut HashMap<QuorumType, QuorumListExtendedInfo> {
        match self {
            PlatformState::V0(v0) => v0.quorums_extended_info_mut(),
        }
    }

    fn current_validator_set_quorum_hash_mut(&mut self) -> &mut QuorumHash {
        match self {
            PlatformState::V0(v0) => v0.current_validator_set_quorum_hash_mut(),
        }
    }

    fn next_validator_set_quorum_hash_mut(&mut self) -> &mut Option<QuorumHash> {
        match self {
            PlatformState::V0(v0) => v0.next_validator_set_quorum_hash_mut(),
        }
    }

    fn validator_sets_mut(&mut self) -> &mut IndexMap<QuorumHash, ValidatorSet> {
        match self {
            PlatformState::V0(v0) => v0.validator_sets_mut(),
        }
    }

    fn full_masternode_list_mut(&mut self) -> &mut BTreeMap<ProTxHash, MasternodeListItem> {
        match self {
            PlatformState::V0(v0) => v0.full_masternode_list_mut(),
        }
    }

    fn hpmn_masternode_list_mut(&mut self) -> &mut BTreeMap<ProTxHash, MasternodeListItem> {
        match self {
            PlatformState::V0(v0) => v0.hpmn_masternode_list_mut(),
        }
    }

    fn initialization_information_mut(&mut self) -> &mut Option<PlatformInitializationState> {
        match self {
            PlatformState::V0(v0) => v0.initialization_information_mut(),
        }
    }

    fn take_next_validator_set_quorum_hash(&mut self) -> Option<QuorumHash> {
        match self {
            PlatformState::V0(v0) => v0.take_next_validator_set_quorum_hash(),
        }
    }
}
