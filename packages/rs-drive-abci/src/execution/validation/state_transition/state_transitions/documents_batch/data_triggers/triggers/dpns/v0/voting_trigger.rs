use crate::error::Error;
use crate::platform_types::platform::PlatformStateRef;
use dpp::block::epoch::Epoch;
use dpp::platform_value::btreemap_extensions::BTreeValueMapHelper;
use dpp::platform_value::Identifier;
use drive::state_transition_action::document::documents_batch::document_transition::DocumentTransitionAction;
use crate::execution::validation::state_transition::documents_batch::data_triggers::{DataTriggerExecutionContext, DataTriggerExecutionResult};
use crate::platform_types::platform_state::v0::PlatformStateV0Methods;

const BLOCKS_SIZE_WINDOW: u32 = 8;
mod property_names {
    pub const TO_USER_ID: &str = "toUserId";
    pub const CORE_HEIGHT_CREATED_AT: &str = "coreHeightCreatedAt";
    pub const CORE_CHAIN_LOCKED_HEIGHT: &str = "coreChainLockedHeight";
}

// TODO: move it to another file
impl PlatformStateRef<'_> {
    /// Returns the current height of the blockchain.
    pub fn epoch(&self) -> Epoch {
        self.state.epoch()
    }
}

/// Runs the name register trigger.
pub fn run_name_register_trigger(
    document_transition: &DocumentTransitionAction,
    context: &DataTriggerExecutionContext<'_>,
    _: Option<&Identifier>,
) -> Result<DataTriggerExecutionResult, Error> {
    let mut data_trigger_execution_result = DataTriggerExecutionResult::default();

    let current_epoch = context.current_epoch();

    if current_epoch.index != 0 {
        // As per spec, the voting for names is only allowed in the first epoch.
        return Ok(data_trigger_execution_result);
    }

    let DocumentTransitionAction::CreateAction(document_create_action) = document_transition else {
        // Not a name registration actions, as it doesn't create a document.
        return Ok(data_trigger_execution_result);
    };

    // TODO: I guess preorders can be created, it's the actual domain document that needs to be
    //  voted on
    if document_create_action.base.document_type_name != "domain" {
        // Not a name registration document.
        return Ok(data_trigger_execution_result);
    };

    println!("document_create_action: {:?}", document_create_action);

    // Check votes here
    data_trigger_execution_result.add_error(DataTriggerActionError::DataTriggerExecutionError {
        data_contract_id: context.data_contract.id.clone(),
        document_transition_id: document_create_action.base.id.clone(),
        message: "Vote didn't happen".to_string(),
        execution_error: "Vote didn't happen".to_string(),
        document_transition: Some(document_transition.clone()),
        owner_id: None,
    });

    Ok(data_trigger_execution_result)
}

#[cfg(test)]
mod test {
    use std::ops::Deref;
    use crate::platform_types::platform::PlatformStateRef;
    use crate::test::helpers::setup::TestPlatformBuilder;
    use dpp::block::block_info::{BlockInfo};
    use dpp::block::epoch::Epoch;
    use dpp::block::extended_block_info::ExtendedBlockInfo;
    use dpp::block::extended_block_info::v0::ExtendedBlockInfoV0;
    use dpp::document::{DocumentV0Getters, DocumentV0Setters};
    use dpp::platform_value::platform_value;
    use dpp::state_transition::documents_batch_transition::document_create_transition::DocumentCreateTransition;
    use dpp::state_transition::documents_batch_transition::document_transition::action_type::DocumentTransitionActionType;
    use dpp::tests::fixtures::{
        get_document_transitions_fixture, get_dpns_data_contract_fixture,
        get_dpns_parent_document_fixture,
        ParentDocumentOptions,
    };
    use drive::state_transition_action::document::documents_batch::document_transition::document_create_transition_action::DocumentCreateTransitionAction;
    use crate::execution::types::state_transition_execution_context::StateTransitionExecutionContext;
    use crate::execution::validation::state_transition::documents_batch::data_triggers::DataTriggerExecutionContext;
    use crate::execution::validation::state_transition::documents_batch::data_triggers::triggers::dpns::v0::voting_trigger::run_name_register_trigger;

    #[test]
    fn should_return_error_if_can_not_get_epoch_info() {
        let platform = TestPlatformBuilder::new()
            .build_with_mock_rpc()
            .set_initial_state_structure();
        let state_read_guard = platform.state.read().unwrap();
        let platform_ref = PlatformStateRef {
            drive: &platform.drive,
            state: &state_read_guard,
            config: &platform.config,
        };
    }

    #[test]
    fn should_prevent_users_from_registering_names_on_epoch_0() {
        let platform = TestPlatformBuilder::new()
            .build_with_mock_rpc()
            .set_initial_state_structure();
        let state_read_guard = platform.state.read().unwrap();
        let platform_ref = PlatformStateRef {
            drive: &platform.drive,
            state: &state_read_guard,
            config: &platform.config,
        };

        let mut domain_document =
            get_dpns_parent_document_fixture(ParentDocumentOptions::default());
        domain_document
            .set(
                super::property_names::CORE_HEIGHT_CREATED_AT,
                platform_value!(10u32),
            )
            .expect("expected to set core height created at");
        let owner_id = &domain_document.owner_id();

        let document_transitions =
            get_document_transitions_fixture([(Action::Create, vec![domain_document])]);
        let document_transition = document_transitions
            .get(0)
            .expect("document transition should be present");

        let document_create_transition = document_transition
            .as_transition_create()
            .expect("expected a document create transition");

        let data_contract = get_dpns_data_contract_fixture(None);

        let transition_execution_context = StateTransitionExecutionContext::default();

        let data_trigger_context = DataTriggerExecutionContext {
            platform: &platform_ref,
            owner_id,
            state_transition_execution_context: &transition_execution_context,
            transaction: None,
        };

        transition_execution_context.enable_dry_run();

        let result = run_name_register_trigger(
            &DocumentCreateTransitionAction::from(document_create_transition).into(),
            &data_trigger_context,
            None,
        )
            .expect("the execution result should be returned");

        assert!(!result.is_valid());

        let data_trigger_error = &result.errors[0];
        match data_trigger_error {
            DataTriggerActionError::DataTriggerExecutionError { message, .. } => {
                assert_eq!(message, "Vote didn't happen");
            }
            _ => {
                panic!("Expected DataTriggerExecutionError");
            }
        }
    }

    #[test]
    fn should_allow_registering_names_after_epoch_0() {
        let platform = TestPlatformBuilder::new()
            .build_with_mock_rpc()
            .set_initial_state_structure();

        {
            let mut state_guard = platform.state.write().unwrap().v0_mut().unwrap();

            state_guard.last_committed_block_info = Some(ExtendedBlockInfo::V0(ExtendedBlockInfoV0 {
                basic_info: BlockInfo {
                    time_ms: 500000,
                    height: 100,
                    core_height: 42,
                    epoch: Epoch::new(1).expect("expected to create epoch"),
                },
                app_hash: platform.drive.grove.root_hash(None).unwrap().unwrap(),
                quorum_hash: [0u8; 32],
                signature: [0u8; 96],
                round: 0,
            }));
        }

        let guard = platform.state.read().unwrap();

        let platform_ref = PlatformStateRef {
            drive: &platform.drive,
            state: guard.deref(),
            config: &platform.config,
        };

        let mut domain_document =
            get_dpns_parent_document_fixture(ParentDocumentOptions::default());
        domain_document
            .set(
                super::property_names::CORE_HEIGHT_CREATED_AT,
                platform_value!(10u32),
            )
            .expect("expected to set core height created at");
        let owner_id = &domain_document.owner_id();

        let document_transitions =
            get_document_transitions_fixture([(DocumentTransitionActionType::Create, vec![domain_document])]);
        let document_transition = document_transitions
            .get(0)
            .expect("document transition should be present");

        let document_create_transition = document_transition
            .as_transition_create()
            .expect("expected a document create transition");

        let data_contract = get_dpns_data_contract_fixture(None);

        let transition_execution_context = StateTransitionExecutionContext::default();

        let data_trigger_context = DataTriggerExecutionContext {
            platform: &platform_ref,
            owner_id,
            state_transition_execution_context: &transition_execution_context,
            transaction: None,
        };

        transition_execution_context.enable_dry_run();
        let heh = match document_create_transition {
            DocumentCreateTransition::V0(tr) => {
                tr
            },
            _ => panic!("Expected to be v0")
        };

        let result = run_name_register_trigger(
            &DocumentCreateTransitionAction::from(document_create_transition).into(),
            &data_trigger_context,
            None,
        )
            .expect("the execution result should be returned");

        assert!(result.is_valid());
    }

    #[test]
    fn should_always_return_valid_results_for_preorder_at_epoch_0() {
        let platform = TestPlatformBuilder::new()
            .build_with_mock_rpc()
            .set_initial_state_structure();
        let state_read_guard = platform.state.read().unwrap();
        let platform_ref = PlatformStateRef {
            drive: &platform.drive,
            state: &state_read_guard,
            config: &platform.config,
        };

        let (mut preorder_document, preorder_salt) =
            get_dpns_preorder_document_fixture(ParentDocumentOptions::default());
        preorder_document
            .set(
                super::property_names::CORE_HEIGHT_CREATED_AT,
                platform_value!(10u32),
            )
            .expect("expected to set core height created at");
        let owner_id = &preorder_document.owner_id();

        let document_transitions =
            get_document_transitions_fixture([(Action::Create, vec![preorder_document])]);
        let document_transition = document_transitions
            .get(0)
            .expect("document transition should be present");

        let document_create_transition = document_transition
            .as_transition_create()
            .expect("expected a document create transition");

        let data_contract = get_dpns_data_contract_fixture(None);

        let transition_execution_context = StateTransitionExecutionContext::default();

        let data_trigger_context = DataTriggerExecutionContext {
            platform: &platform_ref,
            owner_id,
            state_transition_execution_context: &transition_execution_context,
            transaction: None,
        };

        transition_execution_context.enable_dry_run();

        let result = run_name_register_trigger(
            &DocumentCreateTransitionAction::from(document_create_transition).into(),
            &data_trigger_context,
            None,
        )
            .expect("the execution result should be returned");

        assert!(result.is_valid());
    }
}
