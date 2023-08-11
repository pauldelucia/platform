use crate::data_contract::document_type::v0::DocumentTypeV0;
use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::convert::{TryFrom, TryInto};

use crate::consensus::basic::data_contract::{
    DuplicateIndexNameError, InvalidIndexPropertyTypeError, InvalidIndexedPropertyConstraintError,
    SystemPropertyIndexAlreadyPresentError, UndefinedIndexPropertyError,
    UniqueIndicesLimitReachedError,
};
use crate::consensus::ConsensusError;
use crate::data_contract::document_type::array::ArrayItemType;
use crate::data_contract::document_type::index::Index;
use crate::data_contract::document_type::index_level::IndexLevel;
use crate::data_contract::document_type::property::{DocumentProperty, DocumentPropertyType};
use crate::data_contract::document_type::schema::enrich_with_base_schema;
#[cfg(feature = "validation")]
use crate::data_contract::document_type::validation::{
    byte_array_has_no_items_as_parent_validator, pattern_is_valid_regex_validator,
    traversal_validator, validate_data_contract_max_depth,
};
use crate::data_contract::document_type::{property_names, DocumentType};
use crate::data_contract::errors::{DataContractError, StructureError};
use crate::util::json_schema::resolve_uri;
#[cfg(feature = "validation")]
use crate::validation::meta_validators::DOCUMENT_META_SCHEMA_V0;
use crate::version::PlatformVersion;
use crate::ProtocolError;
use platform_value::btreemap_extensions::{BTreeValueMapHelper, BTreeValueRemoveFromMapHelper};
use platform_value::{Identifier, Value, ValueMapHelper};

const UNIQUE_INDEX_LIMIT_V0: usize = 16;
const NOT_ALLOWED_SYSTEM_PROPERTIES: [&str; 1] = ["$id"];

const MAX_INDEXED_STRING_PROPERTY_LENGTH: u16 = 63;
const MAX_INDEXED_BYTE_ARRAY_PROPERTY_LENGTH: u16 = 255;
const MAX_INDEXED_ARRAY_ITEMS: usize = 1024;

impl DocumentTypeV0 {
    // TODO: Split into multiple functions
    pub(crate) fn try_from_schema_v0(
        data_contract_id: Identifier,
        name: &str,
        schema: Value,
        schema_defs: Option<&BTreeMap<String, Value>>,
        default_keeps_history: bool,
        default_mutability: bool,
        platform_version: &PlatformVersion,
    ) -> Result<Self, ProtocolError> {
        // Create a full root JSON Schema from shorten contract document type schema
        let root_schema = enrich_with_base_schema(
            schema.clone(),
            schema_defs.map(|defs| Value::from(defs.clone())),
            &[],
            platform_version,
        )?;

        #[cfg(feature = "validation")]
        {
            // Validate against JSON Schema
            DOCUMENT_META_SCHEMA_V0
                .validate(
                    &root_schema
                        .try_to_validating_json()
                        .map_err(ProtocolError::ValueError)?,
                )
                .map_err(|mut errs| ConsensusError::from(errs.next().unwrap()))?;

            // Validate document schema depth
            let mut result = validate_data_contract_max_depth(&root_schema, platform_version);

            if !result.is_valid() {
                let error = result.errors.remove(0);

                return Err(ProtocolError::ConsensusError(Box::new(error)));
            }

            // TODO: Are we still aiming to use RE2 with linear time complexity to protect from ReDoS attacks?
            //  If not we can remove this validation
            // Validate reg exp compatibility with RE2 and byteArray usage
            result.merge(traversal_validator(
                &root_schema,
                &[
                    pattern_is_valid_regex_validator,
                    byte_array_has_no_items_as_parent_validator,
                ],
                platform_version,
            ));

            if !result.is_valid() {
                let error = result.errors.remove(0);

                return Err(ProtocolError::ConsensusError(Box::new(error)));
            }
        }

        let schema_map = schema.to_map().map_err(|err| {
            ProtocolError::DataContractError(DataContractError::InvalidContractStructure(format!(
                "document schema must be an object: {err}"
            )))
        })?;

        // TODO: These properties aren't defined in JSON meta schema
        // Do documents of this type keep history? (Overrides contract value)
        let documents_keep_history: bool =
            Value::inner_optional_bool_value(schema_map, "documentsKeepHistory")
                .map_err(ProtocolError::ValueError)?
                .unwrap_or(default_keeps_history);

        // Are documents of this type mutable? (Overrides contract value)
        let documents_mutable: bool =
            Value::inner_optional_bool_value(schema_map, "documentsMutable")
                .map_err(ProtocolError::ValueError)?
                .unwrap_or(default_mutability);

        // Extract the properties
        let property_values =
            Value::inner_optional_btree_map(schema_map, property_names::PROPERTIES)?
                .unwrap_or_default();

        // Prepare internal data for efficient querying
        let mut flattened_document_properties: BTreeMap<String, DocumentProperty> = BTreeMap::new();
        let mut document_properties: BTreeMap<String, DocumentProperty> = BTreeMap::new();

        let required_fields = Value::inner_recursive_optional_array_of_strings(
            schema_map,
            "".to_string(),
            property_names::PROPERTIES,
            property_names::REQUIRED,
        );

        // Based on the property name, determine the type
        for (property_key, property_value) in property_values {
            DocumentType::insert_values(
                &mut flattened_document_properties,
                &required_fields,
                None,
                property_key.clone(),
                property_value,
                &root_schema,
                &platform_version
                    .dpp
                    .contract_versions
                    .document_type_versions,
            )?;

            DocumentType::insert_values_nested(
                &mut document_properties,
                &required_fields,
                property_key,
                property_value,
                &root_schema,
                &platform_version
                    .dpp
                    .contract_versions
                    .document_type_versions,
            )?;
        }

        // Initialize indices
        let index_values =
            Value::inner_optional_array_slice_value(schema_map, property_names::INDICES)?;

        let mut index_names: HashSet<String> = HashSet::new();
        let mut unique_indices_count = 0;

        let indices: Vec<Index> = index_values
            .map(|index_values| {
                index_values
                    .iter()
                    .map(|index_value| {
                        let index: Index = index_value
                            .as_map()
                            .ok_or(ProtocolError::DataContractError(
                                DataContractError::InvalidContractStructure(
                                    "index definition is not a map as expected".to_string(),
                                ),
                            ))?
                            .as_slice()
                            .try_into()?;

                        #[cfg(feature = "validation")]
                        {
                            // Unique indices produces significant load on the system during state validation
                            // so we need to limit their number to prevent of spikes and DoS attacks
                            if index.unique {
                                unique_indices_count += 1;
                                if unique_indices_count > UNIQUE_INDEX_LIMIT_V0 {
                                    return Err(ProtocolError::ConsensusError(Box::new(
                                        UniqueIndicesLimitReachedError::new(
                                            name.to_string(),
                                            UNIQUE_INDEX_LIMIT_V0,
                                        )
                                        .into(),
                                    )));
                                }
                            }

                            // Index names must be unique for the document type
                            if !index_names.insert(index.name.to_owned()) {
                                return Err(ProtocolError::ConsensusError(Box::new(
                                    DuplicateIndexNameError::new(name.to_string(), index.name)
                                        .into(),
                                )));
                            }

                            // Validate indexed properties
                            index
                                .properties
                                .iter()
                                .map(|index_property| {
                                    // Do not allow to index already indexed system properties
                                    if NOT_ALLOWED_SYSTEM_PROPERTIES
                                        .contains(&index_property.name.as_str())
                                    {
                                        return Err(ProtocolError::ConsensusError(Box::new(
                                            SystemPropertyIndexAlreadyPresentError::new(
                                                name.to_owned(),
                                                index.name.to_owned(),
                                                index_property.name.to_owned(),
                                            )
                                            .into(),
                                        ))
                                        .into());
                                    }

                                    // Index property must exist
                                    let property_definition = flattened_document_properties
                                        .get(&index_property.name)
                                        .ok_or_else(|| {
                                            ProtocolError::ConsensusError(Box::new(
                                                UndefinedIndexPropertyError::new(
                                                    name.to_owned(),
                                                    index.name.to_owned(),
                                                    index_property.name.to_owned(),
                                                )
                                                .into(),
                                            ))
                                        })?;

                                    // Validate indexed property type
                                    match property_definition.r#type {
                                        // Array and objects aren't supported for indexing yet
                                        DocumentPropertyType::Array(_)
                                        | DocumentPropertyType::Object(_)
                                        | DocumentPropertyType::VariableTypeArray(_) => {
                                            Err(ProtocolError::ConsensusError(Box::new(
                                                InvalidIndexPropertyTypeError::new(
                                                    name.to_owned(),
                                                    index.name.to_owned(),
                                                    index_property.name.to_owned(),
                                                    property_definition.r#type.name(),
                                                )
                                                .into(),
                                            )))
                                        }
                                        // Indexed byte array size must be limited
                                        DocumentPropertyType::ByteArray(_, maybe_max_size)
                                            if maybe_max_size.is_none()
                                                || maybe_max_size.unwrap()
                                                    > MAX_INDEXED_BYTE_ARRAY_PROPERTY_LENGTH =>
                                        {
                                            Err(ProtocolError::ConsensusError(Box::new(
                                                InvalidIndexedPropertyConstraintError::new(
                                                    name.to_owned(),
                                                    index.name.to_owned(),
                                                    index_property.name.to_owned(),
                                                    "maxItems".to_string(),
                                                    format!(
                                                        "should be less or equal {}",
                                                        MAX_INDEXED_BYTE_ARRAY_PROPERTY_LENGTH
                                                    ),
                                                )
                                                .into(),
                                            )))
                                        }
                                        // Indexed string length must be limited
                                        DocumentPropertyType::String(_, maybe_max_length)
                                            if maybe_max_length.is_none()
                                                || maybe_max_length.unwrap()
                                                    > MAX_INDEXED_STRING_PROPERTY_LENGTH =>
                                        {
                                            Err(ProtocolError::ConsensusError(Box::new(
                                                InvalidIndexedPropertyConstraintError::new(
                                                    name.to_owned(),
                                                    index.name.to_owned(),
                                                    index_property.name.to_owned(),
                                                    "maxLength".to_string(),
                                                    format!(
                                                        "should be less or equal {}",
                                                        MAX_INDEXED_STRING_PROPERTY_LENGTH
                                                    ),
                                                )
                                                .into(),
                                            )))
                                        }
                                        _ => Ok(()),
                                    }
                                })
                                .collect::<Result<_, ProtocolError>>()?;
                        }

                        Ok(index)
                    })
                    .collect::<Result<Vec<Index>, ProtocolError>>()
            })
            .transpose()?
            .unwrap_or_default();

        let index_structure =
            IndexLevel::try_from_indices(indices.as_slice(), name, platform_version)?;

        // Collect binary and identifier properties
        let (identifier_paths, binary_paths) = DocumentType::find_identifier_and_binary_paths(
            &document_properties,
            &platform_version
                .dpp
                .contract_versions
                .document_type_versions,
        )?;

        Ok(DocumentTypeV0 {
            name: String::from(name),
            schema,
            indices,
            index_structure,
            flattened_properties: flattened_document_properties,
            properties: document_properties,
            identifier_paths,
            binary_paths,
            required_fields,
            documents_keep_history,
            documents_mutable,
            data_contract_id,
        })
    }
}
