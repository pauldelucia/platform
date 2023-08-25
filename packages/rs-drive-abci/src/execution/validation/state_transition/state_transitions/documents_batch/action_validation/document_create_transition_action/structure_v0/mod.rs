use dpp::consensus::basic::document::{InvalidDocumentTypeError, MissingDocumentTypeError};
use dpp::consensus::ConsensusError;
use dpp::consensus::state::document::document_timestamp_window_violation_error::DocumentTimestampWindowViolationError;
use dpp::consensus::state::document::document_timestamps_mismatch_error::DocumentTimestampsMismatchError;
use dpp::consensus::state::state_error::StateError;
use dpp::data_contract::accessors::v0::DataContractV0Getters;
use dpp::data_contract::document_type::accessors::DocumentTypeV0Getters;
use dpp::data_contract::validation::DataContractValidationMethodsV0;
use dpp::document::extended_document::property_names;
use dpp::identity::TimestampMillis;
use dpp::ProtocolError;
use dpp::validation::{SimpleConsensusValidationResult};
use dpp::validation::block_time_window::validate_time_in_block_time_window::validate_time_in_block_time_window;
use drive::state_transition_action::document::documents_batch::document_transition::document_base_transition_action::DocumentBaseTransitionActionAccessorsV0;
use drive::state_transition_action::document::documents_batch::document_transition::document_create_transition_action::{DocumentCreateTransitionAction, DocumentCreateTransitionActionAccessorsV0};
use dpp::version::PlatformVersion;
use crate::error::Error;
use crate::platform_types::platform::PlatformStateRef;
use crate::platform_types::platform_state::v0::PlatformStateV0Methods;

pub(super) trait DocumentCreateTransitionActionStructureValidationV0 {
    fn validate_structure_v0(
        &self,
        platform: &PlatformStateRef,
        platform_version: &PlatformVersion,
    ) -> Result<SimpleConsensusValidationResult, Error>;
}
impl DocumentCreateTransitionActionStructureValidationV0 for DocumentCreateTransitionAction {
    fn validate_structure_v0(
        &self,
        platform: &PlatformStateRef,
        platform_version: &PlatformVersion,
    ) -> Result<SimpleConsensusValidationResult, Error> {
        let contract_fetch_info = self.base().data_contract_fetch_info();
        let data_contract = &contract_fetch_info.contract;
        // Make sure that the document type is defined in the contract
        let document_type_name = self.base().document_type_name();

        let Some(document_type) = data_contract
            .document_type_optional_for_name(document_type_name) else {
            return Ok(SimpleConsensusValidationResult::new_with_error(
                InvalidDocumentTypeError::new(document_type_name.clone(), data_contract.id()).into(),
            ));
        };

        // Make sure that timestamps are present if required
        let required_fields = document_type.required_fields();

        if required_fields.contains(property_names::CREATED_AT) && self.created_at().is_none() {
            // TODO: Create a special consensus error for this
            return Ok(SimpleConsensusValidationResult::new_with_error(
                MissingDocumentTypeError::new().into(),
            ));
        }

        if required_fields.contains(property_names::UPDATED_AT) && self.updated_at().is_none() {
            // TODO: Create a special consensus error for this
            return Ok(SimpleConsensusValidationResult::new_with_error(
                MissingDocumentTypeError::new().into(),
            ));
        }

        if self.created_at().is_some()
            && self.updated_at().is_some()
            && self.created_at() != self.updated_at()
        {
            return Ok(SimpleConsensusValidationResult::new_with_error(
                DocumentTimestampsMismatchError::new(self.base().id()).into(),
            ));
        }

        let validation_result = check_if_timestamps_are_equal(self);
        if !validation_result.is_valid() {
            return Ok(validation_result);
        }

        // Validate timestamps against block time
        // we do validation here but not in validate state because it's a cheap validation
        // and validate state implements expensive validation only
        let latest_block_time_ms = platform.state.last_block_time_ms();
        let average_block_spacing_ms = platform.config.block_spacing_ms;

        // We do not need to perform these checks on genesis
        if let Some(latest_block_time_ms) = latest_block_time_ms {
            let validation_result = check_created_inside_time_window(
                self,
                latest_block_time_ms,
                average_block_spacing_ms,
                platform_version,
            )?;
            if !validation_result.is_valid() {
                return Ok(validation_result);
            }

            let validation_result = check_updated_inside_time_window(
                self,
                latest_block_time_ms,
                average_block_spacing_ms,
                platform_version,
            )?;

            if !validation_result.is_valid() {
                return Ok(validation_result);
            }
        }

        // Validate user defined properties

        data_contract
            .validate_document_properties(document_type_name, self.data().into(), platform_version)
            .map_err(Error::Protocol)
    }
}

fn check_if_timestamps_are_equal(
    document_transition: &DocumentCreateTransitionAction,
) -> SimpleConsensusValidationResult {
    let mut result = SimpleConsensusValidationResult::default();
    let created_at = document_transition.created_at();
    let updated_at = document_transition.updated_at();

    if created_at.is_some() && updated_at.is_some() && updated_at.unwrap() != created_at.unwrap() {
        result.add_error(ConsensusError::StateError(
            StateError::DocumentTimestampsMismatchError(DocumentTimestampsMismatchError::new(
                document_transition.base().id(),
            )),
        ));
    }

    result
}

fn check_created_inside_time_window(
    document_transition: &DocumentCreateTransitionAction,
    last_block_ts_millis: TimestampMillis,
    average_block_spacing_ms: u64,
    platform_version: &PlatformVersion,
) -> Result<SimpleConsensusValidationResult, Error> {
    let mut result = SimpleConsensusValidationResult::default();
    let created_at = match document_transition.created_at() {
        Some(t) => t,
        None => return Ok(result),
    };

    let window_validation = validate_time_in_block_time_window(
        last_block_ts_millis,
        created_at,
        average_block_spacing_ms,
        platform_version,
    )
    .map_err(|e| Error::Protocol(ProtocolError::NonConsensusError(e)))?;
    if !window_validation.valid {
        result.add_error(ConsensusError::StateError(
            StateError::DocumentTimestampWindowViolationError(
                DocumentTimestampWindowViolationError::new(
                    String::from("createdAt"),
                    document_transition.base().id(),
                    created_at as i64,
                    window_validation.time_window_start as i64,
                    window_validation.time_window_end as i64,
                ),
            ),
        ));
    }
    Ok(result)
}

fn check_updated_inside_time_window(
    document_transition: &DocumentCreateTransitionAction,
    last_block_ts_millis: TimestampMillis,
    average_block_spacing_ms: u64,
    platform_version: &PlatformVersion,
) -> Result<SimpleConsensusValidationResult, Error> {
    let mut result = SimpleConsensusValidationResult::default();
    let updated_at = match document_transition.updated_at() {
        Some(t) => t,
        None => return Ok(result),
    };

    let window_validation = validate_time_in_block_time_window(
        last_block_ts_millis,
        updated_at,
        average_block_spacing_ms,
        platform_version,
    )
    .map_err(|e| Error::Protocol(ProtocolError::NonConsensusError(e)))?;
    if !window_validation.valid {
        result.add_error(ConsensusError::StateError(
            StateError::DocumentTimestampWindowViolationError(
                DocumentTimestampWindowViolationError::new(
                    String::from("updatedAt"),
                    document_transition.base().id(),
                    updated_at as i64,
                    window_validation.time_window_start as i64,
                    window_validation.time_window_end as i64,
                ),
            ),
        ));
    }
    Ok(result)
}
