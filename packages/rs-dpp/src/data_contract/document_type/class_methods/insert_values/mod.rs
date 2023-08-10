use crate::data_contract::document_type::property::DocumentProperty;
use crate::data_contract::document_type::DocumentType;
use crate::version::dpp_versions::DocumentTypeVersions;
use crate::ProtocolError;
use platform_value::Value;
use std::collections::{BTreeMap, BTreeSet};
use crate::data_contract::document_type::v0::DocumentTypeV0;

mod v0;

impl DocumentType {
    pub fn insert_values(
        document_properties: &mut BTreeMap<String, DocumentProperty>,
        known_required: &BTreeSet<String>,
        prefix: Option<String>,
        property_key: String,
        property_value: &Value,
        root_schema: &Value,
        document_type_version: &DocumentTypeVersions,
    ) -> Result<(), ProtocolError> {
        match document_type_version.insert_values {
            0 => DocumentTypeV0::insert_values_v0(
                document_properties,
                known_required,
                prefix,
                property_key,
                property_value,
                root_schema,
            ),
            version => Err(ProtocolError::UnknownVersionMismatch {
                method: "insert_values".to_string(),
                known_versions: vec![0],
                received: version,
            }),
        }
    }
}
