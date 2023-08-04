use crate::data_contract::document_type::document_field::DocumentProperty;
use crate::data_contract::document_type::DocumentType;
use crate::version::dpp_versions::DocumentTypeVersions;
use crate::ProtocolError;
use platform_value::Value;
use std::collections::{BTreeMap, BTreeSet};

mod v0;

impl DocumentType {
    pub fn insert_values_nested(
        document_properties: &mut BTreeMap<String, DocumentProperty>,
        known_required: &BTreeSet<String>,
        property_key: String,
        property_value: &Value,
        schema_defs: &Option<BTreeMap<String, Value>>,
        document_type_version: &DocumentTypeVersions,
    ) -> Result<(), ProtocolError> {
        match document_type_version.insert_values_nested {
            0 => Self::insert_values_nested_v0(
                document_properties,
                known_required,
                property_key,
                property_value,
                schema_defs,
            ),
            version => Err(ProtocolError::UnknownVersionMismatch {
                method: "insert_values_nested".to_string(),
                known_versions: vec![0],
                received: version,
            }),
        }
    }
}
