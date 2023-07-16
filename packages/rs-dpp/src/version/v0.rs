use crate::version::dpp_versions::{
    ContractClassMethodVersions, ContractVersions, DPPVersion, DocumentClassMethodVersions,
    DocumentFeatureVersionBounds, DocumentMethodVersions, DocumentTypeVersions, DocumentVersions,
    IdentityVersions, StateTransitionSerializationVersions,
};
use crate::version::drive_abci_versions::{
    DriveAbciBlockEndMethodVersions, DriveAbciBlockFeeProcessingMethodVersions,
    DriveAbciCoreBasedUpdatesMethodVersions, DriveAbciCoreSubsidyMethodVersions,
    DriveAbciEngineMethodVersions, DriveAbciEpochMethodVersions,
    DriveAbciFeePoolInwardsDistributionMethodVersions,
    DriveAbciFeePoolOutwardsDistributionMethodVersions,
    DriveAbciIdentityCreditWithdrawalMethodVersions, DriveAbciInitializationMethodVersions,
    DriveAbciMasternodeIdentitiesUpdatesMethodVersions, DriveAbciMethodVersions,
    DriveAbciProtocolUpgradeMethodVersions, DriveAbciStateTransitionProcessingMethodVersions,
    DriveAbciStateTransitionValidationVersion, DriveAbciStateTransitionValidationVersions,
    DriveAbciValidationVersions, DriveAbciVersion, DriveAbciWithdrawalsMethodVersions,
};
use crate::version::drive_versions::{
    DriveAssetLockMethodVersions, DriveBalancesMethodVersions, DriveBatchOperationsMethodVersion,
    DriveContractApplyMethodVersions, DriveContractCostsMethodVersions,
    DriveContractGetMethodVersions, DriveContractInsertMethodVersions, DriveContractMethodVersions,
    DriveContractProveMethodVersions, DriveContractUpdateMethodVersions,
    DriveCreditPoolEpochsMethodVersions, DriveCreditPoolMethodVersions,
    DriveCreditPoolPendingEpochRefundsMethodVersions, DriveDocumentDeleteMethodVersions,
    DriveDocumentIndexUniquenessMethodVersions, DriveDocumentInsertMethodVersions,
    DriveDocumentMethodVersions, DriveDocumentUpdateMethodVersions,
    DriveEstimatedCostsMethodVersions, DriveGroveApplyMethodVersions,
    DriveGroveBasicMethodVersions, DriveGroveBatchMethodVersions, DriveGroveCostMethodVersions,
    DriveGroveMethodVersions, DriveIdentityCostEstimationMethodVersions,
    DriveIdentityFetchAttributesMethodVersions, DriveIdentityFetchMethodVersions,
    DriveIdentityFetchPublicKeyHashesMethodVersions,
    DriveIdentityKeyHashesToIdentityInsertMethodVersions, DriveIdentityKeysFetchMethodVersions,
    DriveIdentityKeysInsertMethodVersions, DriveIdentityKeysMethodVersions,
    DriveIdentityKeysProveMethodVersions, DriveIdentityMethodVersions,
    DriveIdentityProveMethodVersions, DriveIdentityUpdateMethodVersions,
    DriveInitializationMethodVersions, DriveMethodVersions, DriveOperationsMethodVersion,
    DrivePlatformSystemMethodVersions, DriveProtocolUpgradeVersions, DriveStructureVersion,
    DriveSystemEstimationCostsMethodVersions, DriveSystemProtocolVersionMethodVersions,
    DriveVersion,
};
use crate::version::protocol_version::{FeatureVersionBounds, PlatformVersion};
use crate::version::{
    AbciStructureVersion, DataContractFactoryVersion, PlatformArchitectureVersion,
    StateTransitionSigningVersion,
};
use std::collections::BTreeMap;

pub(super) const PLATFORM_V1: PlatformVersion = PlatformVersion {
    protocol_version: 0,
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
    state_transition_signing: StateTransitionSigningVersion {
        sign_external: 0,
        sign: 0,
        verify_public_key_is_enabled: 0,
        verify_public_key_level_and_purpose: 0,
    },
    drive: DriveVersion {
        structure: DriveStructureVersion {
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
        methods: DriveMethodVersions {
            initialization: DriveInitializationMethodVersions {
                create_initial_state_structure: 0,
            },
            credit_pools: DriveCreditPoolMethodVersions {
                epochs: DriveCreditPoolEpochsMethodVersions {
                    get_epoch_fee_multiplier: 0,
                    get_epoch_processing_credits_for_distribution: 0,
                    get_epoch_storage_credits_for_distribution: 0,
                    get_epoch_total_credits_for_distribution: 0,
                    get_storage_credits_for_distribution_for_epochs_in_range: 0,
                    get_epoch_start_time: 0,
                    get_epoch_start_block_core_height: 0,
                    get_epoch_start_block_height: 0,
                    get_first_epoch_start_block_info_between_epochs: 0,
                    get_epoch_proposers: 0,
                    get_epochs_proposer_block_count: 0,
                    is_epochs_proposers_tree_empty: 0,
                },
                pending_epoch_refunds: DriveCreditPoolPendingEpochRefundsMethodVersions {
                    add_delete_pending_epoch_refunds_except_specified: 0,
                    fetch_and_add_pending_epoch_refunds_to_collection: 0,
                    fetch_pending_epoch_refunds: 0,
                    add_update_pending_epoch_refunds_operations: 0,
                },
            },
            protocol_upgrade: DriveProtocolUpgradeVersions {
                clear_version_information: 0,
                change_to_new_version_and_clear_version_information: 0,
                fetch_versions_with_counter: 0,
                remove_validators_proposed_app_versions: 0,
                update_validator_proposed_app_version: 0,
            },
            balances: DriveBalancesMethodVersions {
                add_to_system_credits: 0,
                add_to_system_credits_operations: 0,
                remove_from_system_credits: 0,
                remove_from_system_credits_operations: 0,
                calculate_total_credits_balance: 0,
            },
            document: DriveDocumentMethodVersions {
                delete: DriveDocumentDeleteMethodVersions {
                    add_estimation_costs_for_remove_document_to_primary_storage: 0,
                    delete_document_for_contract: 0,
                    delete_document_for_contract_id: 0,
                    delete_document_for_contract_apply_and_add_to_operations: 0,
                    remove_document_from_primary_storage: 0,
                    remove_reference_for_index_level_for_contract_operations: 0,
                    remove_indices_for_index_level_for_contract_operations: 0,
                    remove_indices_for_top_index_level_for_contract_operations: 0,
                    delete_document_for_contract_id_with_named_type_operations: 0,
                    delete_document_for_contract_with_named_type_operations: 0,
                    delete_document_for_contract_operations: 0,
                },
                insert: DriveDocumentInsertMethodVersions {
                    add_document: 0,
                    add_document_for_contract: 0,
                    add_document_for_contract_apply_and_add_to_operations: 0,
                    add_document_for_contract_operations: 0,
                    add_document_to_primary_storage: 0,
                    add_indices_for_index_level_for_contract_operations: 0,
                    add_indices_for_top_index_level_for_contract_operations: 0,
                    add_reference_for_index_level_for_contract_operations: 0,
                    add_serialized_document_for_contract: 0,
                    add_serialized_document_for_contract_id: 0,
                },
                update: DriveDocumentUpdateMethodVersions {
                    add_update_multiple_documents_operations: 0,
                    update_document_for_contract: 0,
                    update_document_for_contract_apply_and_add_to_operations: 0,
                    update_document_for_contract_id: 0,
                    update_document_for_contract_operations: 0,
                    update_document_with_serialization_for_contract: 0,
                    update_serialized_document_for_contract: 0,
                },
                index_uniqueness: DriveDocumentIndexUniquenessMethodVersions {
                    validate_document_uniqueness: 0,
                    validate_document_create_transition_action_uniqueness: 0,
                    validate_document_replace_transition_action_uniqueness: 0,
                    validate_uniqueness_of_data: 0,
                },
            },
            contract: DriveContractMethodVersions {
                prove: DriveContractProveMethodVersions {
                    prove_contract: 0,
                    prove_contract_history: 0,
                    prove_contracts: 0,
                },
                apply: DriveContractApplyMethodVersions {
                    apply_contract: 0,
                    apply_contract_with_serialization: 0,
                },
                insert: DriveContractInsertMethodVersions {
                    add_contract_to_storage: 0,
                    insert_contract: 0,
                },
                update: DriveContractUpdateMethodVersions { update_contract: 0 },
                costs: DriveContractCostsMethodVersions {
                    add_estimation_costs_for_contract_insertion: 0,
                },
                get: DriveContractGetMethodVersions {
                    fetch_contract: 0,
                    fetch_contract_with_history: 0,
                    get_cached_contract_with_fetch_info: 0,
                    get_contract_with_fetch_info: 0,
                    get_contracts_with_fetch_info: 0,
                },
            },
            estimated_costs: DriveEstimatedCostsMethodVersions {
                add_estimation_costs_for_levels_up_to_contract: 0,
                add_estimation_costs_for_levels_up_to_contract_document_type_excluded: 0,
            },
            asset_lock: DriveAssetLockMethodVersions {
                add_asset_lock_outpoint: 0,
                add_estimation_costs_for_adding_asset_lock: 0,
                has_asset_lock_outpoint: 0,
            },
            identity: DriveIdentityMethodVersions {
                fetch: DriveIdentityFetchMethodVersions {
                    public_key_hashes: DriveIdentityFetchPublicKeyHashesMethodVersions {
                        fetch_full_identities_by_unique_public_key_hashes: 0,
                        fetch_full_identity_by_unique_public_key_hash: 0,
                        fetch_identity_id_by_unique_public_key_hash: 0,
                        fetch_identity_ids_by_non_unique_public_key_hash: 0,
                        fetch_identity_ids_by_unique_public_key_hashes: 0,
                        fetch_serialized_full_identity_by_unique_public_key_hash: 0,
                        has_any_of_unique_public_key_hashes: 0,
                        has_non_unique_public_key_hash: 0,
                        has_non_unique_public_key_hash_already_for_identity: 0,
                        has_unique_public_key_hash: 0,
                    },
                    attributes: DriveIdentityFetchAttributesMethodVersions {
                        revision: 0,
                        balance: 0,
                        balance_include_debt: 0,
                        negative_balance: 0,
                    },
                },
                prove: DriveIdentityProveMethodVersions {
                    full_identity: 0,
                    full_identities: 0,
                    prove_full_identities_by_unique_public_key_hashes: 0,
                    prove_full_identity_by_unique_public_key_hash: 0,
                    prove_identity_id_by_unique_public_key_hash: 0,
                    prove_identity_ids_by_unique_public_key_hashes: 0,
                },
                keys: DriveIdentityKeysMethodVersions {
                    fetch: DriveIdentityKeysFetchMethodVersions {
                        fetch_all_current_identity_keys: 0,
                        fetch_all_identity_keys: 0,
                        fetch_identities_all_keys: 0,
                        fetch_identity_keys: 0,
                    },
                    prove: DriveIdentityKeysProveMethodVersions {
                        prove_identities_all_keys: 0,
                        prove_identity_keys: 0,
                    },
                    insert: DriveIdentityKeysInsertMethodVersions {
                        create_key_tree_with_keys: 0,
                        create_new_identity_key_query_trees: 0,
                        insert_key_searchable_references: 0,
                        insert_key_to_storage: 0,
                        insert_new_non_unique_key: 0,
                        insert_new_unique_key: 0,
                        replace_key_in_storage: 0,
                    },
                    insert_key_hash_identity_reference:
                        DriveIdentityKeyHashesToIdentityInsertMethodVersions {
                            add_estimation_costs_for_insert_non_unique_public_key_hash_reference: 0,
                            add_estimation_costs_for_insert_unique_public_key_hash_reference: 0,
                            insert_non_unique_public_key_hash_reference_to_identity: 0,
                            insert_reference_to_non_unique_key: 0,
                            insert_reference_to_unique_key: 0,
                            insert_unique_public_key_hash_reference_to_identity: 0,
                        },
                },
                update: DriveIdentityUpdateMethodVersions {
                    update_identity_revision: 0,
                    initialize_identity_revision: 0,
                    disable_identity_keys: 0,
                    re_enable_identity_keys: 0,
                    add_new_non_unique_keys_to_identity: 0,
                    add_new_unique_keys_to_identity: 0,
                    add_new_keys_to_identity: 0,
                    insert_identity_balance: 0,
                    initialize_negative_identity_balance: 0,
                    update_identity_balance_operation: 0,
                    update_identity_negative_credit: 0,
                    add_to_identity_balance: 0,
                    add_to_previous_balance: 0,
                    apply_balance_change_from_fee_to_identity: 0,
                    remove_from_identity_balance: 0,
                },
                cost_estimation: DriveIdentityCostEstimationMethodVersions {
                    for_authentication_keys_security_level_in_key_reference_tree: 0,
                    for_balances: 0,
                    for_keys_for_identity_id: 0,
                    for_negative_credit: 0,
                    for_purpose_in_key_reference_tree: 0,
                    for_root_key_reference_tree: 0,
                    for_update_revision: 0,
                },
            },
            platform_system: DrivePlatformSystemMethodVersions {
                protocol_version: DriveSystemProtocolVersionMethodVersions {
                    fetch_current_protocol_version: 0,
                    set_current_protocol_version_operations: 0,
                    fetch_next_protocol_version: 0,
                    set_next_protocol_version_operations: 0,
                },
                estimation_costs: DriveSystemEstimationCostsMethodVersions {
                    for_total_system_credits_update: 0,
                },
            },
            operations: DriveOperationsMethodVersion {
                rollback_transaction: 0,
                drop_cache: 0,
                commit_transaction: 0,
                apply_partial_batch_low_level_drive_operations: 0,
                apply_partial_batch_grovedb_operations: 0,
                apply_batch_low_level_drive_operations: 0,
                apply_batch_grovedb_operations: 0,
            },
            batch_operations: DriveBatchOperationsMethodVersion {
                convert_drive_operations_to_grove_operations: 0,
                apply_drive_operations: 0,
            },
        },
        grove_methods: DriveGroveMethodVersions {
            basic: DriveGroveBasicMethodVersions {
                grove_insert: 0,
                grove_insert_empty_tree: 0,
                grove_insert_empty_sum_tree: 0,
                grove_insert_if_not_exists: 0,
                grove_delete: 0,
                grove_get_raw: 0,
                grove_get_raw_optional: 0,
                grove_get_raw_value_u64_from_encoded_var_vec: 0,
                grove_get: 0,
                grove_get_path_query_serialized_results: 0,
                grove_get_path_query: 0,
                grove_get_path_query_with_optional: 0,
                grove_get_raw_path_query_with_optional: 0,
                grove_get_raw_path_query: 0,
                grove_get_proved_path_query: 0,
                grove_get_sum_tree_total_value: 0,
                grove_has_raw: 0,
            },
            batch: DriveGroveBatchMethodVersions {
                batch_insert_empty_tree: 0,
                batch_insert_empty_tree_if_not_exists: 0,
                batch_insert_empty_tree_if_not_exists_check_existing_operations: 0,
                batch_insert: 0,
                batch_insert_if_not_exists: 0,
                batch_insert_if_changed_value: 0,
                batch_delete: 0,
                batch_remove_raw: 0,
                batch_delete_up_tree_while_empty: 0,
                batch_refresh_reference: 0,
            },
            apply: DriveGroveApplyMethodVersions {
                grove_apply_operation: 0,
                grove_apply_batch: 0,
                grove_apply_batch_with_add_costs: 0,
                grove_apply_partial_batch: 0,
                grove_apply_partial_batch_with_add_costs: 0,
            },
            costs: DriveGroveCostMethodVersions {
                grove_batch_operations_costs: 0,
            },
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
        data_contract_factory: DataContractFactoryVersion {
            bounds: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
        },
    },
    drive_abci: DriveAbciVersion {
        methods: DriveAbciMethodVersions {
            engine: DriveAbciEngineMethodVersions { init_chain: 0 },
            initialization: DriveAbciInitializationMethodVersions {
                initial_core_height: 0,
                create_genesis_state: 0,
            },
            core_based_updates: DriveAbciCoreBasedUpdatesMethodVersions {
                update_core_info: 0,
                update_masternode_list: 0,
                update_quorum_info: 0,
                masternode_updates: DriveAbciMasternodeIdentitiesUpdatesMethodVersions {
                    disable_identity_keys: 0,
                    update_masternode_identities: 0,
                    update_operator_identity: 0,
                    update_owner_withdrawal_address: 0,
                    update_voter_identity: 0,
                },
            },
            protocol_upgrade: DriveAbciProtocolUpgradeMethodVersions {
                check_for_desired_protocol_upgrade: 0,
            },
            block_fee_processing: DriveAbciBlockFeeProcessingMethodVersions {
                add_process_epoch_change_operations: 0,
                process_block_fees: 0,
            },
            core_subsidy: DriveAbciCoreSubsidyMethodVersions {
                epoch_core_reward_credits_for_distribution: 0,
            },
            fee_pool_inwards_distribution: DriveAbciFeePoolInwardsDistributionMethodVersions {
                add_distribute_block_fees_into_pools_operations: 0,
                add_distribute_storage_fee_to_epochs_operations: 0,
            },
            fee_pool_outwards_distribution: DriveAbciFeePoolOutwardsDistributionMethodVersions {
                add_distribute_fees_from_oldest_unpaid_epoch_pool_to_proposers_operations: 0,
                add_epoch_pool_to_proposers_payout_operations: 0,
                find_oldest_epoch_needing_payment: 0,
            },
            identity_credit_withdrawal: DriveAbciIdentityCreditWithdrawalMethodVersions {
                build_withdrawal_transactions_from_documents: 0,
                fetch_and_prepare_unsigned_withdrawal_transactions: 0,
                fetch_core_block_transactions: 0,
                pool_withdrawals_into_transactions_queue: 0,
                update_broadcasted_withdrawal_transaction_statuses: 0,
            },
            state_transition_processing: DriveAbciStateTransitionProcessingMethodVersions {
                execute_event: 0,
                process_raw_state_transitions: 0,
                validate_fees_of_event: 0,
            },
            withdrawals: DriveAbciWithdrawalsMethodVersions {
                check_withdrawals: 0,
            },
            epoch: DriveAbciEpochMethodVersions {
                gather_epoch_info: 0,
                get_genesis_time: 0,
            },
            block_end: DriveAbciBlockEndMethodVersions {
                store_ephemeral_state: 0,
                update_state_cache: 0,
                validator_set_update: 0,
            },
        },
        validation_and_processing: DriveAbciValidationVersions {
            state_transitions: DriveAbciStateTransitionValidationVersions {
                identity_create_state_transition: DriveAbciStateTransitionValidationVersion {
                    structure: 0,
                    identity_signatures: 0,
                    state: 0,
                    transform_into_action: 0,
                },
                identity_update_state_transition: DriveAbciStateTransitionValidationVersion {
                    structure: 0,
                    identity_signatures: 0,
                    state: 0,
                    transform_into_action: 0,
                },
                identity_top_up_state_transition: DriveAbciStateTransitionValidationVersion {
                    structure: 0,
                    identity_signatures: 0,
                    state: 0,
                    transform_into_action: 0,
                },
                identity_credit_withdrawal_state_transition:
                    DriveAbciStateTransitionValidationVersion {
                        structure: 0,
                        identity_signatures: 0,
                        state: 0,
                        transform_into_action: 0,
                    },
                identity_credit_transfer_state_transition:
                    DriveAbciStateTransitionValidationVersion {
                        structure: 0,
                        identity_signatures: 0,
                        state: 0,
                        transform_into_action: 0,
                    },
                contract_create_state_transition: DriveAbciStateTransitionValidationVersion {
                    structure: 0,
                    identity_signatures: 0,
                    state: 0,
                    transform_into_action: 0,
                },
                contract_update_state_transition: DriveAbciStateTransitionValidationVersion {
                    structure: 0,
                    identity_signatures: 0,
                    state: 0,
                    transform_into_action: 0,
                },
                documents_batch_state_transition: DriveAbciStateTransitionValidationVersion {
                    structure: 0,
                    identity_signatures: 0,
                    state: 0,
                    transform_into_action: 0,
                },
                document_base_state_transition: DriveAbciStateTransitionValidationVersion {
                    structure: 0,
                    identity_signatures: 0,
                    state: 0,
                    transform_into_action: 0,
                },
                document_create_state_transition: DriveAbciStateTransitionValidationVersion {
                    structure: 0,
                    identity_signatures: 0,
                    state: 0,
                    transform_into_action: 0,
                },
                document_replace_state_transition: DriveAbciStateTransitionValidationVersion {
                    structure: 0,
                    identity_signatures: 0,
                    state: 0,
                    transform_into_action: 0,
                },
                document_delete_state_transition: DriveAbciStateTransitionValidationVersion {
                    structure: 0,
                    identity_signatures: 0,
                    state: 0,
                    transform_into_action: 0,
                },
            },
            process_state_transition: 0,
        },
    },
    dpp: DPPVersion {
        state_transition_serialization_versions: StateTransitionSerializationVersions {
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
            identity_credit_transfer_state_transition: FeatureVersionBounds {
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
            document_create_state_transition: DocumentFeatureVersionBounds {
                bounds: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
            },
            document_replace_state_transition: DocumentFeatureVersionBounds {
                bounds: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
            },
            document_delete_state_transition: DocumentFeatureVersionBounds {
                bounds: FeatureVersionBounds {
                    min_version: 0,
                    max_version: 0,
                    default_current_version: 0,
                },
            },
        },
        contract_versions: ContractVersions {
            contract_serialization_version: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            contract_structure_version: 0,
            document_type_versions: DocumentTypeVersions {
                document_type_structure_version: 0,
                find_identifier_and_binary_paths: 0,
                insert_values: 0,
                insert_values_nested: 0,
                index_for_types: 0,
                unique_id_for_storage: 0,
                unique_id_for_document_field: 0,
                serialize_value_for_key: 0,
                convert_value_to_document: 0,
                max_size: 0,
                estimated_size: 0,
                top_level_indices: 0,
                document_field_for_property: 0,
                document_field_type_for_property: 0,
                field_can_be_null: 0,
                initial_revision: 0,
                requires_revision: 0,
            },
            contract_class_method_versions: ContractClassMethodVersions {
                get_property_definition_by_path: 0,
                get_binary_properties_from_schema: 0,
                get_definitions: 0,
                get_document_types_from_contract: 0,
                get_document_types_from_value: 0,
                get_document_types_from_value_array: 0,
            },
        },
        document_versions: DocumentVersions {
            document_structure_version: 0,
            document_serialization_version: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            },
            extended_document_structure_version: FeatureVersionBounds {
                min_version: 0,
                max_version: 0,
                default_current_version: 0,
            }, //todo (probably should be changed)
            document_method_versions: DocumentMethodVersions {
                hash: 0,
                get_raw_for_contract: 0,
                get_raw_for_document_type: 0,
            },
            document_class_method_versions: DocumentClassMethodVersions {
                get_identifiers_and_binary_paths: 0,
            },
        },
        identity_versions: IdentityVersions {
            identity_structure_version: 0,
        },
    },
};
