use std::collections::BTreeMap;

use platform_serialization::{PlatformDeserialize, PlatformSerialize};

use platform_value::{BinaryData, Bytes32, Error, IntegerReplacementType, ReplacementType, Value};
use serde::{Deserialize, Serialize};

use crate::{Convertible, NonConsensusError, prelude::Identifier, ProtocolError, state_transition::{
    StateTransitionFieldTypes, StateTransitionLike,
    StateTransitionType,
}};

use crate::serialization_traits::{PlatformDeserializable, Signable};
use bincode::{config, Decode, Encode};
use platform_value::btreemap_extensions::BTreeValueRemoveFromMapHelper;
use crate::state_transition::StateTransitionValueConvert;
use crate::state_transition::identity_credit_transfer_transition::{IdentityCreditTransferTransition};
use crate::state_transition::identity_credit_transfer_transition::v0::IdentityCreditTransferTransitionV0;
use crate::state_transition::state_transitions::identity_credit_transfer_transition::fields::*;

impl StateTransitionValueConvert for IdentityCreditTransferTransition {
    fn to_object(&self, skip_signature: bool) -> Result<Value, ProtocolError> {
        match self { 
            IdentityCreditTransferTransition::V0(transition) => {
                let mut value = transition.to_object(skip_signature)?;
                value.insert(STATE_TRANSITION_PROTOCOL_VERSION.to_string(), Value::U16(0))?;
                Ok(value)
            } 
        }
    }

    fn to_canonical_object(&self, skip_signature: bool) -> Result<Value, ProtocolError> {
        match self {
            IdentityCreditTransferTransition::V0(transition) => {
                let mut value = transition.to_canonical_object(skip_signature)?;
                value.insert(STATE_TRANSITION_PROTOCOL_VERSION.to_string(), Value::U16(0))?;
                Ok(value)
            }
        }
    }

    fn to_canonical_cleaned_object(&self, skip_signature: bool) -> Result<Value, ProtocolError> {
        match self {
            IdentityCreditTransferTransition::V0(transition) => {
                let mut value = transition.to_canonical_cleaned_object(skip_signature)?;
                value.insert(STATE_TRANSITION_PROTOCOL_VERSION.to_string(), Value::U16(0))?;
                Ok(value)
            }
        }
    }

    fn to_cleaned_object(&self, skip_signature: bool) -> Result<Value, ProtocolError> {
        match self {
            IdentityCreditTransferTransition::V0(transition) => {
                let mut value = transition.to_cleaned_object(skip_signature)?;
                value.insert(STATE_TRANSITION_PROTOCOL_VERSION.to_string(), Value::U16(0))?;
                Ok(value)
            }
        }
    }

    fn from_raw_object(
        mut raw_object: Value,
    ) -> Result<IdentityCreditTransferTransition, ProtocolError> {
        let version: u8 = raw_object
            .remove_integer(STATE_TRANSITION_PROTOCOL_VERSION)
            .map_err(ProtocolError::ValueError)?;
        match version {
            0 => Ok(IdentityCreditTransferTransitionV0::from_raw_object(raw_object)?.into()),
            n => Err(ProtocolError::UnknownVersionError(format!(
                "Unknown IdentityCreditTransferTransition version {n}"
            ))),
        }
    }

    fn from_value_map(
        mut raw_data_contract_create_transition: BTreeMap<String, Value>,
    ) -> Result<IdentityCreditTransferTransition, ProtocolError> {
        let version: u8 = raw_data_contract_create_transition
            .remove_integer(STATE_TRANSITION_PROTOCOL_VERSION)
            .map_err(ProtocolError::ValueError)?;

        match version {
            0 => Ok(IdentityCreditTransferTransitionV0::from_value_map(
                raw_data_contract_create_transition,
            )?
                .into()),
            n => Err(ProtocolError::UnknownVersionError(format!(
                "Unknown IdentityCreditTransferTransition version {n}"
            ))),
        }
    }


    fn clean_value(value: &mut Value) -> Result<(), ProtocolError> {
        let version: u8 = value
            .get_integer(STATE_TRANSITION_PROTOCOL_VERSION)
            .map_err(ProtocolError::ValueError)?;

        match version {
            0 => {
                IdentityCreditTransferTransitionV0::clean_value(
                    value,
                )
            },
            n => Err(ProtocolError::UnknownVersionError(format!(
                "Unknown IdentityCreditTransferTransition version {n}"
            ))),
        }
    }
}