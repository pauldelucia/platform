use crate::version::protocol_version::{
    DriveStructureVersion, FeatureVersionBounds, PlatformVersion, StateTransitionVersion,
};
use crate::version::{AbciStructureVersion, PlatformArchitectureVersion};

pub(super) const PLATFORM_V1: PlatformVersion = PlatformVersion {
    protocol_version: 0,
    document: FeatureVersionBounds {
        min_version: 0,
        max_version: 0,
        default_current_version: 0,
    },
    extended_document: FeatureVersionBounds {
        min_version: 0,
        max_version: 0,
        default_current_version: 0,
    },
    contract: FeatureVersionBounds {
        min_version: 0,
        max_version: 0,
        default_current_version: 0,
    },
    identity: FeatureVersionBounds {
        min_version: 0,
        max_version: 0,
        default_current_version: 0,
    },
    proofs: FeatureVersionBounds {
        min_version: 0,
        max_version: 0,
        default_current_version: 0,
    },
    costs: FeatureVersionBounds {
        min_version: 0,
        max_version: 0,
        default_current_version: 0,
    },
    state_transitions: StateTransitionVersion {
        identity_create_state_transition: FeatureVersionBounds {
            min_version: 0,
            max_version: 0,
            default_current_version: 0,
        },
        identity_update_state_transition: FeatureVersionBounds {
            min_version: 0,
            max_version: 0,
            default_current_version: 0,
        },
        identity_top_up_state_transition: FeatureVersionBounds {
            min_version: 0,
            max_version: 0,
            default_current_version: 0,
        },
        identity_credit_withdrawal_state_transition: FeatureVersionBounds {
            min_version: 0,
            max_version: 0,
            default_current_version: 0,
        },
        contract_create_state_transition: FeatureVersionBounds {
            min_version: 0,
            max_version: 0,
            default_current_version: 0,
        },
        contract_update_state_transition: FeatureVersionBounds {
            min_version: 0,
            max_version: 0,
            default_current_version: 0,
        },
        documents_batch_state_transition: FeatureVersionBounds {
            min_version: 0,
            max_version: 0,
            default_current_version: 0,
        },
        document_base_state_transition: FeatureVersionBounds {
            min_version: 0,
            max_version: 0,
            default_current_version: 0,
        },
    },
    drive_structure: DriveStructureVersion {
        document_indexes: FeatureVersionBounds {
            min_version: 0,
            max_version: 0,
            default_current_version: 0,
        },
        identity_indexes: FeatureVersionBounds {
            min_version: 0,
            max_version: 0,
            default_current_version: 0,
        },
        pools: FeatureVersionBounds {
            min_version: 0,
            max_version: 0,
            default_current_version: 0,
        },
    },
    abci_structure: AbciStructureVersion {
        extended_block_info: FeatureVersionBounds {
            min_version: 0,
            max_version: 0,
            default_current_version: 0,
        },
    },
    platform_architecture: PlatformArchitectureVersion {
        data_contract_factory: FeatureVersionBounds {
            min_version: 0,
            max_version: 0,
            default_current_version: 0,
        },
    },
};
