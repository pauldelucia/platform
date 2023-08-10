#[cfg(feature = "full")]
#[cfg(test)]
mod tests {
    use dpp::data_contract::document_type::{DocumentType, DocumentTypeRef, Index, IndexProperty};
    use dpp::platform_value::{platform_value, Identifier};
    use dpp::util::cbor_serializer;
    use serde_json::json;

    use crate::drive::config::DriveConfig;
    use crate::error::{query::QuerySyntaxError, Error};
    use crate::query::DriveQuery;
    use dpp::data_contract::document_type::accessors::DocumentTypeV0Getters;
    use dpp::data_contract::document_type::v0::DocumentTypeV0;
    use dpp::data_contract::DataContract;
    use dpp::tests::fixtures::get_dpns_data_contract_fixture;
    use dpp::version::PlatformVersion;

    fn construct_indexed_document_type() -> DocumentType {
        let platform_version = PlatformVersion::latest();

        let schema = platform_value!({
            "type": "object",
            "indices": [
                {
                    "name": "a",
                    "properties": [
                        {
                            "name": "a",
                            "asc": true
                        }
                    ],
                    "unique": false
                },
                {
                    "name": "b",
                    "properties": [
                        {
                            "name": "b",
                            "asc": false
                        }
                    ],
                    "unique": false
                },
                {
                    "name": "c",
                    "properties": [
                        {
                            "name": "b",
                            "asc": false
                        },
                        {
                            "name": "a",
                            "asc": false
                        }
                    ],
                    "unique": false
                },
                {
                    "name": "d",
                    "properties": [
                        {
                            "name": "b",
                            "asc": false
                        },
                        {
                            "name": "a",
                            "asc": false
                        },
                        {
                            "name": "d",
                            "asc": false
                        }
                    ],
                    "unique": false
                }
            ],
            "properties": {
                "a": {
                    "type": "string",
                    "maxLength": 10,
                },
                "b": {
                    "type": "string",
                    "maxLength": 10,
                },
                "c": {
                    "type": "string",
                    "maxLength": 10,
                },
                "d": {
                    "type": "string",
                    "maxLength": 10,
                }
            },
            "additionalProperties": false,
        });

        DocumentType::try_from_schema(
            Identifier::random(),
            "indexed_type",
            schema,
            None,
            false,
            false,
            platform_version,
        )
        .expect("expected to create a document type")
    }

    #[test]
    fn test_find_best_index() {
        let document_type = construct_indexed_document_type();
        let contract = get_dpns_data_contract_fixture(None, 1).data_contract_owned();

        let platform_version = PlatformVersion::latest();

        let query_value = json!({
            "where": [
                ["a", "==", "1"],
                ["b", "==", "2"],
            ]
        });
        let where_cbor = cbor_serializer::serializable_value_to_cbor(&query_value, None)
            .expect("expected to serialize to cbor");
        let query = DriveQuery::from_cbor(
            where_cbor.as_slice(),
            &contract,
            document_type.as_ref(),
            &DriveConfig::default(),
        )
        .expect("query should be valid");
        let index = query
            .find_best_index(platform_version)
            .expect("expected to find index");
        assert_eq!(index, document_type.indices().get(2).unwrap());

        let query_value = json!({
            "where": [
                ["a", "==", "1"],
            ]
        });
        let where_cbor = cbor_serializer::serializable_value_to_cbor(&query_value, None)
            .expect("expected to serialize to cbor");
        let query = DriveQuery::from_cbor(
            where_cbor.as_slice(),
            &contract,
            document_type.as_ref(),
            &DriveConfig::default(),
        )
        .expect("query should be valid");
        let index = query
            .find_best_index(platform_version)
            .expect("expected to find index");
        assert_eq!(index, document_type.indices().get(0).unwrap());
    }

    #[test]
    fn test_find_best_index_error() {
        let document_type = construct_indexed_document_type();
        let contract = get_dpns_data_contract_fixture(None, 1).data_contract_owned();

        let platform_version = PlatformVersion::latest();

        let query_value = json!({
            "where": [
                ["c", "==", "1"]
            ]
        });
        let where_cbor = cbor_serializer::serializable_value_to_cbor(&query_value, None)
            .expect("expected to serialize to cbor");
        let query = DriveQuery::from_cbor(
            where_cbor.as_slice(),
            &contract,
            document_type.as_ref(),
            &DriveConfig::default(),
        )
        .expect("query should be valid");
        let error = query
            .find_best_index(platform_version)
            .expect_err("expected to not find index");
        assert!(
            matches!(error, Error::Query(QuerySyntaxError::WhereClauseOnNonIndexedProperty(message)) if message == "query must be for valid indexes")
        )
    }
}
