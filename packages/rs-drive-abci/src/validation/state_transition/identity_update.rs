use dpp::identity::state_transition::identity_update_transition::identity_update_transition::IdentityUpdateTransition;

use super::ValidateStateTransition;

impl ValidateStateTransition for IdentityUpdateTransition {
    fn validate_type(&self) -> Result<dpp::validation::SimpleValidationResult, dpp::ProtocolError> {
        todo!()
    }

    fn validate_signature(
        &self,
    ) -> Result<dpp::validation::SimpleValidationResult, dpp::ProtocolError> {
        todo!()
    }

    fn validate_key_signature(
        &self,
    ) -> Result<dpp::validation::SimpleValidationResult, dpp::ProtocolError> {
        todo!()
    }

    fn validate_state(
        &self,
        drive: &drive::drive::Drive,
    ) -> Result<
        dpp::validation::ValidationResult<dpp::state_transition::StateTransitionAction>,
        dpp::ProtocolError,
    > {
        todo!()
    }
}
