use dpp::identity::PartialIdentity;
use dpp::prelude::ConsensusValidationResult;
use dpp::{
    consensus::basic::{data_contract::InvalidDataContractIdError},
    data_contract::{
        state_transition::data_contract_create_transition::DataContractCreateTransitionAction,
    },
    validation::SimpleConsensusValidationResult,
};
use dpp::{
    data_contract::{
        generate_data_contract_id,
        state_transition::data_contract_create_transition::{
            DataContractCreateTransition,
        },
    },
};
use dpp::consensus::state::data_contract::data_contract_already_present_error::DataContractAlreadyPresentError;
use dpp::consensus::state::state_error::StateError;
use dpp::state_transition::StateTransitionAction;
use dpp::data_contract::state_transition::data_contract_create_transition::validation::state::validate_data_contract_create_transition_basic::DATA_CONTRACT_CREATE_SCHEMA_VALIDATOR;
use drive::grovedb::TransactionArg;

use crate::error::Error;
use crate::platform::PlatformRef;
use crate::rpc::core::CoreRPCLike;
use crate::validation::state_transition::context::ValidationDataShareContext;
use crate::validation::state_transition::key_validation::validate_state_transition_identity_signature;
use crate::validation::state_transition::StateTransitionValidation;

use super::common::validate_schema;

impl StateTransitionValidation for DataContractCreateTransition {
    fn validate_structure<C: CoreRPCLike>(
        &self,
        _platform: &PlatformRef<C>,
        _context: &mut ValidationDataShareContext,
        _tx: TransactionArg,
    ) -> Result<SimpleConsensusValidationResult, Error> {
        let result = validate_schema(&DATA_CONTRACT_CREATE_SCHEMA_VALIDATOR, self);
        if !result.is_valid() {
            return Ok(result);
        }

        //todo: re-enable version validation
        // // Validate protocol version
        // let protocol_version_validator = ProtocolVersionValidator::default();
        // let result = protocol_version_validator
        //     .validate(self.protocol_version)
        //     .expect("TODO: again, how this will ever fail, why do we even need a validator trait");
        // if !result.is_valid() {
        //     return Ok(result);
        // }
        //
        // // Validate data contract
        // let data_contract_validator =
        //     DataContractValidator::new(Arc::new(protocol_version_validator)); // ffs
        // let result = data_contract_validator
        //     .validate(&(self.data_contract.to_cleaned_object().expect("TODO")))?;
        // if !result.is_valid() {
        //     return Ok(result);
        // }

        // Validate data contract id
        let generated_id = generate_data_contract_id(self.data_contract.owner_id, self.entropy);
        if generated_id.as_slice() != self.data_contract.id.as_ref() {
            return Ok(SimpleConsensusValidationResult::new_with_error(
                InvalidDataContractIdError::new(
                    generated_id.to_vec(),
                    self.data_contract.id.as_ref().to_owned(),
                ),
            ));
        }

        self.data_contract
            .validate_structure()
            .map_err(Error::Protocol)
    }

    fn validate_identity_and_signatures<C: CoreRPCLike>(
        &self,
        platform: &PlatformRef<C>,
        _context: &mut ValidationDataShareContext,
        tx: TransactionArg,
    ) -> Result<ConsensusValidationResult<Option<PartialIdentity>>, Error> {
        Ok(
            validate_state_transition_identity_signature(platform.drive, self, false, tx)?
                .map(Some),
        )
    }

    fn validate_state<C: CoreRPCLike>(
        &self,
        platform: &PlatformRef<C>,
        context: &mut ValidationDataShareContext,
        tx: TransactionArg,
    ) -> Result<ConsensusValidationResult<StateTransitionAction>, Error> {
        let drive = platform.drive;
        // Data contract shouldn't exist
        if drive
            .get_contract_with_fetch_info_and_fee(
                self.data_contract.id.to_buffer(),
                None,
                false,
                tx,
            )?
            .1
            .is_some()
        {
            Ok(ConsensusValidationResult::new_with_errors(vec![
                StateError::DataContractAlreadyPresentError(DataContractAlreadyPresentError::new(
                    self.data_contract.id.to_owned(),
                ))
                .into(),
            ]))
        } else {
            self.transform_into_action(platform, context, tx)
        }
    }

    fn transform_into_action<C: CoreRPCLike>(
        &self,
        _platform: &PlatformRef<C>,
        _context: &ValidationDataShareContext,
        _tx: TransactionArg,
    ) -> Result<ConsensusValidationResult<StateTransitionAction>, Error> {
        let action: StateTransitionAction =
            Into::<DataContractCreateTransitionAction>::into(self).into();
        Ok(action.into())
    }
}
