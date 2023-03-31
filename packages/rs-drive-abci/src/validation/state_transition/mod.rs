mod data_contract_create;
mod data_contract_update;
mod documents_batch;
mod identity_create;
mod identity_credit_withdrawal;
mod identity_update;
mod identity_top_up;

use dpp::state_transition::{StateTransition, StateTransitionAction};
use dpp::validation::{SimpleValidationResult, ValidationResult};
use dpp::ProtocolError;
use drive::drive::Drive;

use crate::platform::Platform;

pub fn validate_state_transition<C, V>(
    platform: &Platform<C>,
    state_transition: V,
) -> Result<ValidationResult<StateTransitionAction>, ProtocolError>
where
    V: ValidateStateTransition,
{
    let result = state_transition.validate_type()?;
    if !result.is_valid() {
        return Ok(ValidationResult::<StateTransitionAction>::new_with_errors(
            result.errors,
        ));
    }
    let result = state_transition.validate_signature()?;
    if !result.is_valid() {
        return Ok(ValidationResult::<StateTransitionAction>::new_with_errors(
            result.errors,
        ));
    }
    let result = state_transition.validate_key_signature()?;
    if !result.is_valid() {
        return Ok(ValidationResult::<StateTransitionAction>::new_with_errors(
            result.errors,
        ));
    }

    state_transition.validate_state(&platform.drive)
}

pub trait ValidateStateTransition {
    fn validate_type(&self) -> Result<SimpleValidationResult, ProtocolError>;
    fn validate_signature(&self) -> Result<SimpleValidationResult, ProtocolError>;
    fn validate_key_signature(&self) -> Result<SimpleValidationResult, ProtocolError>;
    fn validate_state(
        &self,
        drive: &Drive,
    ) -> Result<ValidationResult<StateTransitionAction>, ProtocolError>;
}

impl ValidateStateTransition for StateTransition {
    fn validate_type(&self) -> Result<SimpleValidationResult, ProtocolError> {
        match self {
            StateTransition::DataContractCreate(st) => st.validate_type(),
            StateTransition::DataContractUpdate(st) => st.validate_type(),
            StateTransition::IdentityCreate(st) => st.validate_type(),
            StateTransition::IdentityUpdate(st) => st.validate_type(),
            StateTransition::IdentityTopUp(st) => st.validate_type(),
            StateTransition::IdentityCreditWithdrawal(st) => st.validate_type(),
            StateTransition::DocumentsBatch(st) => st.validate_type(),
        }
    }

    fn validate_signature(&self) -> Result<SimpleValidationResult, ProtocolError> {
        match self {
            StateTransition::DataContractCreate(st) => st.validate_signature(),
            StateTransition::DataContractUpdate(st) => st.validate_signature(),
            StateTransition::IdentityCreate(st) => st.validate_signature(),
            StateTransition::IdentityUpdate(st) => st.validate_signature(),
            StateTransition::IdentityTopUp(st) => st.validate_signature(),
            StateTransition::IdentityCreditWithdrawal(st) => st.validate_signature(),
            StateTransition::DocumentsBatch(st) => st.validate_signature(),
        }
    }

    fn validate_key_signature(&self) -> Result<SimpleValidationResult, ProtocolError> {
        match self {
            StateTransition::DataContractCreate(st) => st.validate_key_signature(),
            StateTransition::DataContractUpdate(st) => st.validate_key_signature(),
            StateTransition::IdentityCreate(st) => st.validate_key_signature(),
            StateTransition::IdentityUpdate(st) => st.validate_key_signature(),
            StateTransition::IdentityTopUp(st) => st.validate_key_signature(),
            StateTransition::IdentityCreditWithdrawal(st) => st.validate_key_signature(),
            StateTransition::DocumentsBatch(st) => st.validate_key_signature(),
        }
    }

    fn validate_state(
        &self,
        drive: &Drive,
    ) -> Result<ValidationResult<StateTransitionAction>, ProtocolError> {
        match self {
            StateTransition::DataContractCreate(st) => st.validate_state(drive),
            StateTransition::DataContractUpdate(st) => st.validate_state(drive),
            StateTransition::IdentityCreate(st) => st.validate_state(drive),
            StateTransition::IdentityUpdate(st) => st.validate_state(drive),
            StateTransition::IdentityTopUp(st) => st.validate_state(drive),
            StateTransition::IdentityCreditWithdrawal(st) => st.validate_state(drive),
            StateTransition::DocumentsBatch(st) => st.validate_state(drive),
        }
    }
}
