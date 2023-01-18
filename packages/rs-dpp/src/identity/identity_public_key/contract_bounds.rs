use crate::data_contract::DataContract;
use crate::identifier::Identifier;
use crate::identity::contract_bounds::ContractBounds::{
    NoContractBounds, SingleContract, SingleContractDocumentType,
};
use crate::identity::identity_public_key::CborValue;
use crate::util::cbor_value::{CborCanonicalMap, CborMapExtension};
use crate::ProtocolError;
use anyhow::bail;
use bincode::{deserialize, serialize};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::format;

pub type ContractBoundsType = u8;

/// A contract bounds is the bounds that the key has influence on.
/// For authentication keys the bounds mean that the keys can only be used to sign
/// within the specified contract.
/// For encryption decryption this tells clients to only use these keys for specific
/// contracts.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ContractBounds {
    /// the key is for general use
    NoContractBounds,
    /// this key can only be used within a specific contract
    SingleContract(Identifier),
    /// this key can only be used within a specific contract and for a specific document type
    SingleContractDocumentType(Identifier, String),
}

impl ContractBounds {
    /// Creates a new contract bounds for the key
    pub fn new_from_type(
        contract_bounds_type: u8,
        identifier: Vec<u8>,
        document_type: String,
    ) -> Result<Self, ProtocolError> {
        Ok(match contract_bounds_type {
            0 => NoContractBounds,
            1 => SingleContract(Identifier::from_bytes(identifier.as_slice())?),
            2 => SingleContractDocumentType(
                Identifier::from_bytes(identifier.as_slice())?,
                document_type,
            ),
            _ => {
                return Err(ProtocolError::InvalidKeyContractBoundsError(format!(
                    "unrecognized contract bounds type: {}",
                    contract_bounds_type
                )))
            }
        })
    }

    /// Gets the contract bounds type
    pub fn contract_bounds_type(&self) -> ContractBoundsType {
        match self {
            NoContractBounds => 0,
            SingleContract(_) => 1,
            SingleContractDocumentType(_, _) => 2,
        }
    }

    /// Gets the identifier
    pub fn identifier(&self) -> Option<&Identifier> {
        match self {
            NoContractBounds => None,
            SingleContract(identifier) => Some(identifier),
            SingleContractDocumentType(identifier, _) => Some(identifier),
        }
    }

    /// Gets the document type
    pub fn document_type(&self) -> Option<&String> {
        match self {
            NoContractBounds => None,
            SingleContract(_) => None,
            SingleContractDocumentType(_, document_type) => Some(document_type),
        }
    }

    /// Gets the cbor value
    pub fn to_cbor_value(&self) -> CborValue {
        let mut pk_map = CborCanonicalMap::new();

        let contract_bounds_type = self.contract_bounds_type();
        pk_map.insert("type", self.contract_bounds_type());

        if contract_bounds_type > 0 {
            pk_map.insert("identifier", self.identifier().unwrap().to_buffer_vec());
        }

        if contract_bounds_type == 2 {
            pk_map.insert("documentType", self.document_type().unwrap().clone());
        }
        pk_map.to_value_sorted()
    }

    /// Gets the cbor value
    pub fn from_cbor_value(cbor_value: &CborValue) -> Result<Self, ProtocolError> {
        let key_value_map = cbor_value.as_map().ok_or_else(|| {
            ProtocolError::DecodingError(String::from(
                "Expected identity public key to be a key value map",
            ))
        })?;

        let contract_bounds_type =
            key_value_map.as_u8("type", "Contract bounds must have a type")?;
        let contract_bounds_identifier = if contract_bounds_type > 0 {
            key_value_map.as_vec(
                "identifier",
                "Contract bounds must have an identifier if it is not type 0",
            )?
        } else {
            vec![]
        };
        let contract_bounds_document_type = if contract_bounds_type == 2 {
            key_value_map.as_string(
                "documentType",
                "Contract bounds must have a document type if it is type 2",
            )?
        } else {
            String::new()
        };
        ContractBounds::new_from_type(
            contract_bounds_type,
            contract_bounds_identifier,
            contract_bounds_document_type,
        )
    }
}
