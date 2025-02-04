use std::sync::Arc;

use chrono::Utc;

use crate::consensus::basic::BasicError;
use crate::consensus::state::state_error::StateError;
use crate::consensus::test_consensus_error::TestConsensusError;
use crate::state_transition::state_transition_execution_context::StateTransitionExecutionContext;
use crate::{
    block_time_window::validate_time_in_block_time_window::BLOCK_TIME_WINDOW_MILLIS,
    consensus::ConsensusError,
    identity::{
        state_transition::identity_update_transition::{
            identity_update_transition::IdentityUpdateTransition,
            validate_identity_update_transition_state::IdentityUpdateTransitionStateValidator,
        },
        validation::MockTPublicKeysValidator,
        Purpose, SecurityLevel,
    },
    prelude::Identity,
    state_repository::MockStateRepositoryLike,
    state_transition::StateTransitionLike,
    tests::{
        fixtures::{get_identity_update_transition_fixture, identity_fixture},
        utils::get_state_error_from_result,
    },
    validation::SimpleConsensusValidationResult,
    NativeBlsModule,
};

struct TestData {
    identity: Identity,
    validate_public_keys_mock: MockTPublicKeysValidator,
    state_repository_mock: MockStateRepositoryLike,
    state_transition: IdentityUpdateTransition,
    header_timestamp: u64,
}

fn setup_test() -> TestData {
    let bls = NativeBlsModule::default();
    let identity = identity_fixture();
    let header_timestamp = Utc::now().timestamp_millis() as u64;

    let mut validate_public_keys_mock = MockTPublicKeysValidator::new();
    validate_public_keys_mock
        .expect_validate_keys()
        .returning(|_| Ok(Default::default()));

    let mut state_repository_mock = MockStateRepositoryLike::new();
    let identity_to_return = identity.clone();

    state_repository_mock
        .expect_fetch_identity()
        .returning(move |_, _| Ok(Some(identity_to_return.clone())));
    state_repository_mock
        .expect_fetch_latest_platform_block_time()
        .returning(move || Ok(header_timestamp));

    let mut state_transition = get_identity_update_transition_fixture();
    state_transition.set_revision(identity.get_revision() + 1);
    state_transition.set_public_key_ids_to_disable(vec![]);
    state_transition.set_public_keys_disabled_at(None);

    let private_key =
        hex::decode("9b67f852093bc61cea0eeca38599dbfba0de28574d2ed9b99d10d33dc1bde7b2").unwrap();
    state_transition
        .sign_by_private_key(
            &private_key,
            crate::identity::KeyType::ECDSA_SECP256K1,
            &bls,
        )
        .expect("transition should be signed");

    TestData {
        identity,
        state_repository_mock,
        validate_public_keys_mock,
        state_transition,
        header_timestamp,
    }
}

#[tokio::test]
async fn should_return_invalid_identity_revision_error_if_new_revision_is_not_incremented_by_1() {
    let TestData {
        identity,
        state_repository_mock,
        validate_public_keys_mock,
        mut state_transition,
        ..
    } = setup_test();
    state_transition.set_revision(identity.get_revision() + 2);

    let validator = IdentityUpdateTransitionStateValidator::new(
        Arc::new(state_repository_mock),
        Arc::new(validate_public_keys_mock),
    );
    let execution_context = StateTransitionExecutionContext::default();

    let result = validator
        .validate(&state_transition, &execution_context)
        .await
        .expect("the validation result should be returned");
    let state_error = get_state_error_from_result(&result, 0);

    assert!(matches!(
        state_error,
        StateError::InvalidIdentityRevisionError(e) if  {
            e.identity_id() ==  state_transition.get_identity_id()  &&
            e.current_revision() == &0
        }
    ));
}

#[tokio::test]
async fn should_return_identity_public_key_is_read_only_error_if_disabling_public_key_is_read_only()
{
    let TestData {
        mut identity,
        validate_public_keys_mock,
        mut state_transition,
        ..
    } = setup_test();
    identity.public_keys.get_mut(&0).unwrap().read_only = true;
    state_transition.set_public_key_ids_to_disable(vec![0]);

    let identity_to_return = identity.clone();
    let mut state_repository_mock = MockStateRepositoryLike::new();
    state_repository_mock
        .expect_fetch_identity()
        .returning(move |_, _| Ok(Some(identity_to_return.clone())));

    let validator = IdentityUpdateTransitionStateValidator::new(
        Arc::new(state_repository_mock),
        Arc::new(validate_public_keys_mock),
    );
    let execution_context = StateTransitionExecutionContext::default();

    let result = validator
        .validate(&state_transition, &execution_context)
        .await
        .expect("the validation result should be returned");
    let state_error = get_state_error_from_result(&result, 0);

    assert!(matches!(
        state_error,
        StateError::IdentityPublicKeyIsReadOnlyError(e) if   e.public_key_index() == 0
    ));
}

#[tokio::test]
async fn should_return_error_if_disabling_public_key_is_already_disabled() {
    let TestData {
        mut identity,
        validate_public_keys_mock,
        mut state_transition,
        ..
    } = setup_test();
    identity.public_keys.get_mut(&0).unwrap().disabled_at =
        Some(Utc::now().timestamp_millis() as u64);
    state_transition.set_public_key_ids_to_disable(vec![0]);

    let identity_to_return = identity.clone();
    let mut state_repository_mock = MockStateRepositoryLike::new();
    state_repository_mock
        .expect_fetch_identity()
        .returning(move |_, _| Ok(Some(identity_to_return.clone())));

    let validator = IdentityUpdateTransitionStateValidator::new(
        Arc::new(state_repository_mock),
        Arc::new(validate_public_keys_mock),
    );
    let execution_context = StateTransitionExecutionContext::default();

    let result = validator
        .validate(&state_transition, &execution_context)
        .await
        .expect("the validation result should be returned");
    let state_error = get_state_error_from_result(&result, 0);

    assert!(matches!(
        state_error,
        StateError::IdentityPublicKeyIsDisabledError(e) if   e.public_key_index() == 0
    ));
}

#[tokio::test]
async fn should_return_invalid_result_if_disabled_at_has_violated_time_window() {
    let TestData {
        validate_public_keys_mock,
        state_repository_mock,
        mut state_transition,
        ..
    } = setup_test();
    state_transition.set_public_key_ids_to_disable(vec![1]);
    state_transition.set_public_keys_disabled_at(Some(
        Utc::now().timestamp_millis() as u64 - (BLOCK_TIME_WINDOW_MILLIS * 2),
    ));

    let validator = IdentityUpdateTransitionStateValidator::new(
        Arc::new(state_repository_mock),
        Arc::new(validate_public_keys_mock),
    );
    let execution_context = StateTransitionExecutionContext::default();

    let result = validator
        .validate(&state_transition, &execution_context)
        .await
        .expect("the validation result should be returned");
    let state_error = get_state_error_from_result(&result, 0);

    assert!(matches!(
        state_error,
        StateError::IdentityPublicKeyDisabledAtWindowViolationError { .. }
    ));
}

#[tokio::test]
async fn should_throw_invalid_identity_public_key_id_error_if_identity_does_not_contain_public_key_with_disabling_id(
) {
    let TestData {
        validate_public_keys_mock,
        state_repository_mock,
        mut state_transition,
        ..
    } = setup_test();
    state_transition.set_public_key_ids_to_disable(vec![3]);
    state_transition.set_public_keys_disabled_at(Some(Utc::now().timestamp_millis() as u64));

    let validator = IdentityUpdateTransitionStateValidator::new(
        Arc::new(state_repository_mock),
        Arc::new(validate_public_keys_mock),
    );
    let execution_context = StateTransitionExecutionContext::default();

    let result = validator
        .validate(&state_transition, &execution_context)
        .await
        .expect("the validation result should be returned");
    let state_error = get_state_error_from_result(&result, 0);

    assert!(matches!(
        state_error,
        StateError::InvalidIdentityPublicKeyIdError(e) if  {
            e.id() == 3
        }
    ));
}

#[tokio::test]
async fn should_pass_when_disabling_public_key() {
    let TestData {
        validate_public_keys_mock,
        state_repository_mock,
        mut state_transition,
        ..
    } = setup_test();
    state_transition.set_public_key_ids_to_disable(vec![1]);
    state_transition.set_public_keys_disabled_at(Some(Utc::now().timestamp_millis() as u64));

    let validator = IdentityUpdateTransitionStateValidator::new(
        Arc::new(state_repository_mock),
        Arc::new(validate_public_keys_mock),
    );
    let execution_context = StateTransitionExecutionContext::default();

    let result = validator
        .validate(&state_transition, &execution_context)
        .await
        .expect("the validation result should be returned");
    assert!(result.is_valid());
}

#[tokio::test]
async fn should_pass_when_adding_public_key() {
    let TestData {
        validate_public_keys_mock,
        state_repository_mock,
        mut state_transition,
        ..
    } = setup_test();
    state_transition.set_public_key_ids_to_disable(vec![]);
    state_transition.set_public_keys_disabled_at(None);

    let validator = IdentityUpdateTransitionStateValidator::new(
        Arc::new(state_repository_mock),
        Arc::new(validate_public_keys_mock),
    );
    let execution_context = StateTransitionExecutionContext::default();

    let result = validator
        .validate(&state_transition, &execution_context)
        .await
        .expect("the validation result should be returned");
    assert!(result.is_valid(), "{:?}", result.errors);
}

#[tokio::test]
async fn should_pass_when_both_adding_and_disabling_public_keys() {
    let TestData {
        validate_public_keys_mock,
        state_repository_mock,
        mut state_transition,
        ..
    } = setup_test();
    state_transition.set_public_key_ids_to_disable(vec![1]);
    state_transition.set_public_keys_disabled_at(Some(Utc::now().timestamp_millis() as u64));

    let validator = IdentityUpdateTransitionStateValidator::new(
        Arc::new(state_repository_mock),
        Arc::new(validate_public_keys_mock),
    );
    let execution_context = StateTransitionExecutionContext::default();

    let result = validator
        .validate(&state_transition, &execution_context)
        .await
        .expect("the validation result should be returned");
    assert!(result.is_valid());
}

#[tokio::test]
async fn should_validate_purpose_and_security_level() {
    let TestData {
        validate_public_keys_mock,
        mut state_transition,
        mut identity,
        header_timestamp,
        ..
    } = setup_test();

    // the identity after transition must contain at least one
    // key with: purpose: AUTHENTICATION AND security level: MASTER
    identity
        .get_public_keys_mut()
        .iter_mut()
        .for_each(|(_, k)| {
            k.purpose = Purpose::ENCRYPTION;
            k.security_level = SecurityLevel::CRITICAL;
        });
    state_transition
        .get_public_keys_to_add_mut()
        .iter_mut()
        .for_each(|k| {
            k.purpose = Purpose::ENCRYPTION;
            k.security_level = SecurityLevel::CRITICAL;
        });

    let identity_to_return = identity.clone();
    let mut state_repository_mock = MockStateRepositoryLike::new();
    state_repository_mock
        .expect_fetch_identity()
        .returning(move |_, _| Ok(Some(identity_to_return.clone())));

    state_repository_mock
        .expect_fetch_latest_platform_block_time()
        .returning(move || Ok(header_timestamp));

    let validator = IdentityUpdateTransitionStateValidator::new(
        Arc::new(state_repository_mock),
        Arc::new(validate_public_keys_mock),
    );
    let execution_context = StateTransitionExecutionContext::default();

    let result = validator
        .validate(&state_transition, &execution_context)
        .await
        .expect("the validation result should be returned");

    assert!(matches!(
        result.errors[0],
        ConsensusError::BasicError(BasicError::MissingMasterPublicKeyError(_))
    ));
}

#[tokio::test]
async fn should_validate_pubic_keys_to_add() {
    let TestData {
        state_repository_mock,
        state_transition,
        ..
    } = setup_test();
    let mut validate_public_keys_mock = MockTPublicKeysValidator::new();
    let some_consensus_error = ConsensusError::TestConsensusError(TestConsensusError::new("test"));
    let validation_result =
        SimpleConsensusValidationResult::new_with_errors(vec![some_consensus_error]);
    validate_public_keys_mock
        .expect_validate_keys()
        .return_once(|_| Ok(validation_result));

    let validator = IdentityUpdateTransitionStateValidator::new(
        Arc::new(state_repository_mock),
        Arc::new(validate_public_keys_mock),
    );
    let execution_context = StateTransitionExecutionContext::default();

    let result = validator
        .validate(&state_transition, &execution_context)
        .await
        .expect("the validation result should be returned");

    assert!(matches!(
        result.errors[0],
        ConsensusError::TestConsensusError(_)
    ));
}

#[tokio::test]
async fn should_return_valid_result_on_dry_run() {
    let TestData {
        validate_public_keys_mock,
        mut state_transition,
        ..
    } = setup_test();
    state_transition.set_public_key_ids_to_disable(vec![3]);
    state_transition.set_public_keys_disabled_at(Some(Utc::now().timestamp_millis() as u64));
    let execution_context = StateTransitionExecutionContext::default().with_dry_run();

    let mut state_repository_mock = MockStateRepositoryLike::new();
    state_repository_mock
        .expect_fetch_identity()
        .return_once(|_, _| Ok(None));

    let validator = IdentityUpdateTransitionStateValidator::new(
        Arc::new(state_repository_mock),
        Arc::new(validate_public_keys_mock),
    );
    let result = validator
        .validate(&state_transition, &execution_context)
        .await
        .expect("the validation result should be returned");
    assert!(result.is_valid());
}
