// MIT LICENSE
//
// Copyright (c) 2021 Dash Core Group
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
//

//! Update Documents.
//!
//! This modules implements functions in Drive relevant to updating Documents.
//!

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use dpp::data_contract::document_type::DocumentType;

use grovedb::batch::key_info::KeyInfo;
use grovedb::batch::key_info::KeyInfo::KnownKey;
use grovedb::batch::KeyInfoPath;
use grovedb::{Element, EstimatedLayerInformation, TransactionArg};

use crate::contract::Contract;
use crate::drive::batch::drive_op_batch::{
    DocumentOperation, DocumentOperationsForContractDocumentType, UpdateOperationInfo,
};
use crate::drive::batch::{DocumentOperationType, DriveOperation};
use crate::drive::defaults::CONTRACT_DOCUMENTS_PATH_HEIGHT;
use crate::drive::document::{
    contract_document_type_path,
    contract_documents_keeping_history_primary_key_path_for_document_id,
    contract_documents_primary_key_path, make_document_reference,
};
use crate::drive::flags::StorageFlags;
use crate::drive::object_size_info::DocumentInfo::{
    DocumentOwnedInfo, DocumentRefAndSerialization, DocumentRefInfo,
};
use dpp::document::Document;

use crate::drive::object_size_info::PathKeyElementInfo::PathKeyRefElement;
use crate::drive::object_size_info::{
    DocumentAndContractInfo, DriveKeyInfo, OwnedDocumentInfo, PathKeyInfo,
};
use crate::drive::Drive;
use crate::error::drive::DriveError;
use crate::error::Error;
use crate::fee::calculate_fee;
use crate::fee::op::LowLevelDriveOperation;

use crate::drive::object_size_info::DriveKeyInfo::{Key, KeyRef, KeySize};
use crate::error::document::DocumentError;
use dpp::block::block_info::BlockInfo;

use crate::drive::grove_operations::{
    BatchDeleteUpTreeApplyType, BatchInsertApplyType, BatchInsertTreeApplyType, DirectQueryType,
    QueryType,
};

use crate::fee::result::FeeResult;
use dpp::prelude::DataContract;

impl Drive {
    /// Updates a serialized document given a contract CBOR and returns the associated fee.
    pub fn update_document_for_contract_cbor(
        &self,
        serialized_document: &[u8],
        contract_cbor: &[u8],
        document_type_name: &str,
        owner_id: Option<[u8; 32]>,
        block_info: BlockInfo,
        apply: bool,
        storage_flags: Option<Cow<StorageFlags>>,
        transaction: TransactionArg,
    ) -> Result<FeeResult, Error> {
        let contract = Contract::from_cbor(contract_cbor)?;

        let document = Document::from_cbor(serialized_document, None, owner_id)?;

        let document_type = contract.document_type_for_name(document_type_name)?;

        let reserialized_document = document.serialize(document_type)?;

        self.update_document_with_serialization_for_contract(
            &document,
            reserialized_document.as_slice(),
            &contract,
            document_type_name,
            owner_id,
            block_info,
            apply,
            storage_flags,
            transaction,
        )
    }

    /// Updates a serialized document given a contract id and returns the associated fee.
    pub fn update_document_for_contract_id(
        &self,
        serialized_document: &[u8],
        contract_id: [u8; 32],
        document_type: &str,
        owner_id: Option<[u8; 32]>,
        block_info: BlockInfo,
        apply: bool,
        storage_flags: Option<Cow<StorageFlags>>,
        transaction: TransactionArg,
    ) -> Result<FeeResult, Error> {
        let mut drive_operations: Vec<LowLevelDriveOperation> = vec![];
        let estimated_costs_only_with_layer_info = if apply {
            None::<HashMap<KeyInfoPath, EstimatedLayerInformation>>
        } else {
            Some(HashMap::new())
        };

        let contract_fetch_info = self
            .get_contract_with_fetch_info_and_add_to_operations(
                contract_id,
                Some(&block_info.epoch),
                true,
                transaction,
                &mut drive_operations,
            )?
            .ok_or(Error::Document(DocumentError::ContractNotFound))?;

        let contract = &contract_fetch_info.contract;

        let document = Document::from_cbor(serialized_document, None, owner_id)?;

        let document_info =
            DocumentRefAndSerialization((&document, serialized_document, storage_flags));

        let document_type = contract.document_type_for_name(document_type)?;

        self.update_document_for_contract_apply_and_add_to_operations(
            DocumentAndContractInfo {
                owned_document_info: OwnedDocumentInfo {
                    document_info,
                    owner_id,
                },
                contract,
                document_type,
            },
            &block_info,
            estimated_costs_only_with_layer_info,
            transaction,
            &mut drive_operations,
        )?;

        let fees = calculate_fee(None, Some(drive_operations), &block_info.epoch)?;

        Ok(fees)
    }

    /// Updates a serialized document and returns the associated fee.
    pub fn update_serialized_document_for_contract(
        &self,
        serialized_document: &[u8],
        contract: &Contract,
        document_type: &str,
        owner_id: Option<[u8; 32]>,
        block_info: BlockInfo,
        apply: bool,
        storage_flags: Option<Cow<StorageFlags>>,
        transaction: TransactionArg,
    ) -> Result<FeeResult, Error> {
        let document = Document::from_cbor(serialized_document, None, owner_id)?;

        self.update_document_with_serialization_for_contract(
            &document,
            serialized_document,
            contract,
            document_type,
            owner_id,
            block_info,
            apply,
            storage_flags,
            transaction,
        )
    }

    /// Updates a document and returns the associated fee.
    pub fn update_document_for_contract(
        &self,
        document: &Document,
        contract: &Contract,
        document_type: &DocumentType,
        owner_id: Option<[u8; 32]>,
        block_info: BlockInfo,
        apply: bool,
        storage_flags: Option<Cow<StorageFlags>>,
        transaction: TransactionArg,
    ) -> Result<FeeResult, Error> {
        let mut drive_operations: Vec<LowLevelDriveOperation> = vec![];
        let estimated_costs_only_with_layer_info = if apply {
            None::<HashMap<KeyInfoPath, EstimatedLayerInformation>>
        } else {
            Some(HashMap::new())
        };

        let document_info = DocumentRefInfo((document, storage_flags));

        self.update_document_for_contract_apply_and_add_to_operations(
            DocumentAndContractInfo {
                owned_document_info: OwnedDocumentInfo {
                    document_info,
                    owner_id,
                },
                contract,
                document_type,
            },
            &block_info,
            estimated_costs_only_with_layer_info,
            transaction,
            &mut drive_operations,
        )?;
        let fees = calculate_fee(None, Some(drive_operations), &block_info.epoch)?;
        Ok(fees)
    }

    /// Updates a document and returns the associated fee.
    pub fn update_document_with_serialization_for_contract(
        &self,
        document: &Document,
        serialized_document: &[u8],
        contract: &Contract,
        document_type_name: &str,
        owner_id: Option<[u8; 32]>,
        block_info: BlockInfo,
        apply: bool,
        storage_flags: Option<Cow<StorageFlags>>,
        transaction: TransactionArg,
    ) -> Result<FeeResult, Error> {
        let mut drive_operations: Vec<LowLevelDriveOperation> = vec![];
        let estimated_costs_only_with_layer_info = if apply {
            None::<HashMap<KeyInfoPath, EstimatedLayerInformation>>
        } else {
            Some(HashMap::new())
        };

        let document_type = contract.document_type_for_name(document_type_name)?;

        let document_info =
            DocumentRefAndSerialization((document, serialized_document, storage_flags));

        self.update_document_for_contract_apply_and_add_to_operations(
            DocumentAndContractInfo {
                owned_document_info: OwnedDocumentInfo {
                    document_info,
                    owner_id,
                },
                contract,
                document_type,
            },
            &block_info,
            estimated_costs_only_with_layer_info,
            transaction,
            &mut drive_operations,
        )?;
        let fees = calculate_fee(None, Some(drive_operations), &block_info.epoch)?;
        Ok(fees)
    }

    /// Updates a document.
    pub(crate) fn update_document_for_contract_apply_and_add_to_operations(
        &self,
        document_and_contract_info: DocumentAndContractInfo,
        block_info: &BlockInfo,
        mut estimated_costs_only_with_layer_info: Option<
            HashMap<KeyInfoPath, EstimatedLayerInformation>,
        >,
        transaction: TransactionArg,
        drive_operations: &mut Vec<LowLevelDriveOperation>,
    ) -> Result<(), Error> {
        let batch_operations = self.update_document_for_contract_operations(
            document_and_contract_info,
            block_info,
            &mut None,
            &mut estimated_costs_only_with_layer_info,
            transaction,
        )?;
        self.apply_batch_low_level_drive_operations(
            estimated_costs_only_with_layer_info,
            transaction,
            batch_operations,
            drive_operations,
        )
    }

    /// Gathers operations for updating a document.
    pub(crate) fn update_document_for_contract_operations(
        &self,
        document_and_contract_info: DocumentAndContractInfo,
        block_info: &BlockInfo,
        previous_batch_operations: &mut Option<&mut Vec<LowLevelDriveOperation>>,
        estimated_costs_only_with_layer_info: &mut Option<
            HashMap<KeyInfoPath, EstimatedLayerInformation>,
        >,
        transaction: TransactionArg,
    ) -> Result<Vec<LowLevelDriveOperation>, Error> {
        let mut batch_operations: Vec<LowLevelDriveOperation> = vec![];
        if !document_and_contract_info.document_type.documents_mutable {
            return Err(Error::Drive(DriveError::UpdatingReadOnlyImmutableDocument(
                "documents for this contract are not mutable",
            )));
        }

        // If we are going for estimated costs do an add instead as it always worse than an update
        if document_and_contract_info
            .owned_document_info
            .document_info
            .is_document_size()
            || estimated_costs_only_with_layer_info.is_some()
        {
            return self.add_document_for_contract_operations(
                document_and_contract_info,
                true, // we say we should override as this skips an unnecessary check
                block_info,
                previous_batch_operations,
                estimated_costs_only_with_layer_info,
                transaction,
            );
        }

        let contract = document_and_contract_info.contract;
        let document_type = document_and_contract_info.document_type;
        let owner_id = document_and_contract_info.owned_document_info.owner_id;
        let Some((document, storage_flags)) = document_and_contract_info.owned_document_info.document_info.get_borrowed_document_and_storage_flags() else {
                return Err(Error::Drive(DriveError::CorruptedCodeExecution("must have document and storage flags")));
            };
        // we need to construct the path for documents on the contract
        // the path is
        //  * Document and Contract root tree
        //  * Contract ID recovered from document
        //  * 0 to signify Documents and not Contract
        let contract_document_type_path =
            contract_document_type_path(contract.id.as_bytes(), document_type.name.as_str());

        let contract_documents_primary_key_path = contract_documents_primary_key_path(
            contract.id.as_bytes(),
            document_type.name.as_str(),
        );

        let document_reference = make_document_reference(
            document,
            document_and_contract_info.document_type,
            storage_flags,
        );

        // next we need to get the old document from storage
        let old_document_element = if document_type.documents_keep_history {
            let contract_documents_keeping_history_primary_key_path_for_document_id =
                contract_documents_keeping_history_primary_key_path_for_document_id(
                    contract.id.as_bytes(),
                    document_type.name.as_str(),
                    document.id.as_slice(),
                );
            // When keeping document history the 0 is a reference that points to the current value
            // O is just on one byte, so we have at most one hop of size 1 (1 byte)
            self.grove_get(
                (&contract_documents_keeping_history_primary_key_path_for_document_id).into(),
                &[0],
                QueryType::StatefulQuery,
                transaction,
                &mut batch_operations,
            )?
        } else {
            self.grove_get_raw(
                (&contract_documents_primary_key_path).into(),
                document.id.as_slice(),
                DirectQueryType::StatefulDirectQuery,
                transaction,
                &mut batch_operations,
            )?
        };

        // we need to store the document for it's primary key
        // we should be overriding if the document_type does not have history enabled
        self.add_document_to_primary_storage(
            &document_and_contract_info,
            block_info,
            true,
            estimated_costs_only_with_layer_info,
            transaction,
            &mut batch_operations,
        )?;

        let old_document_info = if let Some(old_document_element) = old_document_element {
            if let Element::Item(old_serialized_document, element_flags) = old_document_element {
                let document =
                    Document::from_bytes(old_serialized_document.as_slice(), document_type)?;
                let storage_flags = StorageFlags::map_some_element_flags_ref(&element_flags)?;
                Ok(DocumentOwnedInfo((document, storage_flags.map(Cow::Owned))))
            } else {
                Err(Error::Drive(DriveError::CorruptedDocumentNotItem(
                    "old document is not an item",
                )))
            }?
        } else {
            return Err(Error::Drive(DriveError::UpdatingDocumentThatDoesNotExist(
                "document being updated does not exist",
            )));
        };

        let mut batch_insertion_cache: HashSet<Vec<Vec<u8>>> = HashSet::new();
        // fourth we need to store a reference to the document for each index
        for index in &document_type.indices {
            // at this point the contract path is to the contract documents
            // for each index the top index component will already have been added
            // when the contract itself was created
            let mut index_path: Vec<Vec<u8>> = contract_document_type_path
                .iter()
                .map(|&x| Vec::from(x))
                .collect();
            let top_index_property = index.properties.get(0).ok_or(Error::Drive(
                DriveError::CorruptedContractIndexes("invalid contract indices"),
            ))?;
            index_path.push(Vec::from(top_index_property.name.as_bytes()));

            // with the example of the dashpay contract's first index
            // the index path is now something like Contracts/ContractID/Documents(1)/$ownerId
            let document_top_field = document
                .get_raw_for_document_type(&top_index_property.name, document_type, owner_id)?
                .unwrap_or_default();

            let old_document_top_field = old_document_info
                .get_raw_for_document_type(&top_index_property.name, document_type, owner_id, None)?
                .unwrap_or_default();

            // if we are not applying that means we are trying to get worst case costs
            // which would entail a change on every index
            let mut change_occurred_on_index = match &old_document_top_field {
                DriveKeyInfo::Key(k) => &document_top_field != k,
                DriveKeyInfo::KeyRef(k) => document_top_field.as_slice() != *k,
                DriveKeyInfo::KeySize(_) => {
                    // we should assume true in this worst case cost scenario
                    true
                }
            };

            if change_occurred_on_index {
                // here we are inserting an empty tree that will have a subtree of all other index properties
                let mut qualified_path = index_path.clone();
                qualified_path.push(document_top_field.clone());

                if !batch_insertion_cache.contains(&qualified_path) {
                    let inserted = self.batch_insert_empty_tree_if_not_exists(
                        PathKeyInfo::PathKeyRef::<0>((
                            index_path.clone(),
                            document_top_field.as_slice(),
                        )),
                        storage_flags,
                        BatchInsertTreeApplyType::StatefulBatchInsertTree,
                        transaction,
                        previous_batch_operations,
                        &mut batch_operations,
                    )?;
                    if inserted {
                        batch_insertion_cache.insert(qualified_path);
                    }
                }
            }

            let mut all_fields_null = document_top_field.is_empty();

            let mut old_index_path: Vec<DriveKeyInfo> = index_path
                .iter()
                .map(|path_item| DriveKeyInfo::Key(path_item.clone()))
                .collect();
            // we push the actual value of the index path
            index_path.push(document_top_field);
            // the index path is now something like Contracts/ContractID/Documents(1)/$ownerId/<ownerId>

            old_index_path.push(old_document_top_field);

            for i in 1..index.properties.len() {
                let index_property = index.properties.get(i).ok_or(Error::Drive(
                    DriveError::CorruptedContractIndexes("invalid contract indices"),
                ))?;

                let document_index_field = document
                    .get_raw_for_document_type(&index_property.name, document_type, owner_id)?
                    .unwrap_or_default();

                let old_document_index_field = old_document_info
                    .get_raw_for_document_type(&index_property.name, document_type, owner_id, None)?
                    .unwrap_or_default();

                // if we are not applying that means we are trying to get worst case costs
                // which would entail a change on every index
                change_occurred_on_index |= match &old_document_index_field {
                    DriveKeyInfo::Key(k) => &document_index_field != k,
                    DriveKeyInfo::KeyRef(k) => document_index_field != *k,
                    DriveKeyInfo::KeySize(_) => {
                        // we should assume true in this worst case cost scenario
                        true
                    }
                };

                if change_occurred_on_index {
                    // here we are inserting an empty tree that will have a subtree of all other index properties

                    let mut qualified_path = index_path.clone();
                    qualified_path.push(index_property.name.as_bytes().to_vec());

                    if !batch_insertion_cache.contains(&qualified_path) {
                        let inserted = self.batch_insert_empty_tree_if_not_exists(
                            PathKeyInfo::PathKeyRef::<0>((
                                index_path.clone(),
                                index_property.name.as_bytes(),
                            )),
                            storage_flags,
                            BatchInsertTreeApplyType::StatefulBatchInsertTree,
                            transaction,
                            previous_batch_operations,
                            &mut batch_operations,
                        )?;
                        if inserted {
                            batch_insertion_cache.insert(qualified_path);
                        }
                    }
                }

                index_path.push(Vec::from(index_property.name.as_bytes()));
                old_index_path.push(DriveKeyInfo::Key(Vec::from(index_property.name.as_bytes())));

                // Iteration 1. the index path is now something like Contracts/ContractID/Documents(1)/$ownerId/<ownerId>/toUserId
                // Iteration 2. the index path is now something like Contracts/ContractID/Documents(1)/$ownerId/<ownerId>/toUserId/<ToUserId>/accountReference

                if change_occurred_on_index {
                    // here we are inserting an empty tree that will have a subtree of all other index properties

                    let mut qualified_path = index_path.clone();
                    qualified_path.push(document_index_field.clone());

                    if !batch_insertion_cache.contains(&qualified_path) {
                        let inserted = self.batch_insert_empty_tree_if_not_exists(
                            PathKeyInfo::PathKeyRef::<0>((
                                index_path.clone(),
                                document_index_field.as_slice(),
                            )),
                            storage_flags,
                            BatchInsertTreeApplyType::StatefulBatchInsertTree,
                            transaction,
                            previous_batch_operations,
                            &mut batch_operations,
                        )?;
                        if inserted {
                            batch_insertion_cache.insert(qualified_path);
                        }
                    }
                }

                all_fields_null &= document_index_field.is_empty();

                // we push the actual value of the index path, both for the new and the old
                index_path.push(document_index_field);
                old_index_path.push(old_document_index_field);
                // Iteration 1. the index path is now something like Contracts/ContractID/Documents(1)/$ownerId/<ownerId>/toUserId/<ToUserId>/
                // Iteration 2. the index path is now something like Contracts/ContractID/Documents(1)/$ownerId/<ownerId>/toUserId/<ToUserId>/accountReference/<accountReference>
            }

            if change_occurred_on_index {
                // we first need to delete the old values
                // unique indexes will be stored under key "0"
                // non unique indices should have a tree at key "0" that has all elements based off of primary key

                let mut key_info_path = KeyInfoPath::from_vec(
                    old_index_path
                        .into_iter()
                        .map(|key_info| match key_info {
                            Key(key) => KnownKey(key),
                            KeyRef(key_ref) => KnownKey(key_ref.to_vec()),
                            KeySize(key_info) => key_info,
                        })
                        .collect::<Vec<KeyInfo>>(),
                );

                if !index.unique {
                    key_info_path.push(KnownKey(vec![0]));

                    // here we should return an error if the element already exists
                    self.batch_delete_up_tree_while_empty(
                        key_info_path,
                        document.id.as_slice(),
                        Some(CONTRACT_DOCUMENTS_PATH_HEIGHT),
                        BatchDeleteUpTreeApplyType::StatefulBatchDelete {
                            is_known_to_be_subtree_with_sum: Some((false, false)),
                        },
                        transaction,
                        previous_batch_operations,
                        &mut batch_operations,
                    )?;
                } else {
                    // here we should return an error if the element already exists
                    self.batch_delete_up_tree_while_empty(
                        key_info_path,
                        &[0],
                        Some(CONTRACT_DOCUMENTS_PATH_HEIGHT),
                        BatchDeleteUpTreeApplyType::StatefulBatchDelete {
                            is_known_to_be_subtree_with_sum: Some((false, false)),
                        },
                        transaction,
                        previous_batch_operations,
                        &mut batch_operations,
                    )?;
                }

                // unique indexes will be stored under key "0"
                // non unique indices should have a tree at key "0" that has all elements based off of primary key
                if !index.unique || all_fields_null {
                    // here we are inserting an empty tree that will have a subtree of all other index properties
                    self.batch_insert_empty_tree_if_not_exists(
                        PathKeyInfo::PathKeyRef::<0>((index_path.clone(), &[0])),
                        storage_flags,
                        BatchInsertTreeApplyType::StatefulBatchInsertTree,
                        transaction,
                        previous_batch_operations,
                        &mut batch_operations,
                    )?;
                    index_path.push(vec![0]);

                    // here we should return an error if the element already exists
                    self.batch_insert(
                        PathKeyRefElement::<0>((
                            index_path,
                            document.id.as_slice(),
                            document_reference.clone(),
                        )),
                        &mut batch_operations,
                    )?;
                } else {
                    // in one update you can't insert an element twice, so need to check the cache
                    // here we should return an error if the element already exists
                    let inserted = self.batch_insert_if_not_exists(
                        PathKeyRefElement::<0>((index_path, &[0], document_reference.clone())),
                        BatchInsertApplyType::StatefulBatchInsert,
                        transaction,
                        &mut batch_operations,
                    )?;
                    if !inserted {
                        return Err(Error::Drive(DriveError::CorruptedContractIndexes(
                            "index already exists",
                        )));
                    }
                }
            } else {
                // no change occurred on index, we need to refresh the references

                // We can only trust the reference content has not changed if there are no storage flags
                let trust_refresh_reference = storage_flags.is_none();

                // unique indexes will be stored under key "0"
                // non unique indices should have a tree at key "0" that has all elements based off of primary key
                if !index.unique || all_fields_null {
                    index_path.push(vec![0]);

                    // here we should return an error if the element already exists
                    self.batch_refresh_reference(
                        index_path,
                        document.id.to_vec(),
                        document_reference.clone(),
                        trust_refresh_reference,
                        &mut batch_operations,
                    )?;
                } else {
                    self.batch_refresh_reference(
                        index_path,
                        vec![0],
                        document_reference.clone(),
                        trust_refresh_reference,
                        &mut batch_operations,
                    )?;
                }
            }
        }
        Ok(batch_operations)
    }

    /// Add update multiple documents operations
    pub fn add_update_multiple_documents_operations<'a>(
        &self,
        documents: &'a [Document],
        data_contract: &'a DataContract,
        document_type: &'a DocumentType,
        drive_operation_types: &mut Vec<DriveOperation<'a>>,
    ) {
        let operations: Vec<DocumentOperation> = documents
            .iter()
            .map(|document| {
                DocumentOperation::UpdateOperation(UpdateOperationInfo {
                    document,
                    serialized_document: None,
                    owner_id: None,
                    storage_flags: None,
                })
            })
            .collect();

        if !operations.is_empty() {
            drive_operation_types.push(DriveOperation::DocumentOperation(
                DocumentOperationType::MultipleDocumentOperationsForSameContractDocumentType {
                    document_operations: DocumentOperationsForContractDocumentType {
                        operations,
                        contract: data_contract,
                        document_type,
                    },
                },
            ));
        }
    }
}

#[cfg(feature = "full")]
#[cfg(test)]
mod tests {
    use dpp::document::fetch_and_validate_data_contract::DataContractFetcherAndValidator;
    use dpp::state_repository::MockStateRepositoryLike;
    use grovedb::TransactionArg;
    use std::default::Default;
    use std::option::Option::None;
    use std::sync::Arc;

    use dpp::data_contract::validation::data_contract_validator::DataContractValidator;
    use dpp::data_contract::DataContractFactory;
    use dpp::document::document_factory::DocumentFactory;
    use dpp::document::document_validator::DocumentValidator;

    use dpp::platform_value::{platform_value, Identifier, Value};

    use dpp::util::cbor_serializer;
    use dpp::version::{ProtocolVersionValidator, COMPATIBILITY_MAP, LATEST_VERSION};
    use rand::Rng;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use tempfile::TempDir;

    use super::*;
    use crate::drive::config::DriveConfig;
    use crate::drive::flags::StorageFlags;
    use crate::drive::object_size_info::DocumentAndContractInfo;
    use crate::drive::object_size_info::DocumentInfo::DocumentRefInfo;
    use crate::drive::{defaults, Drive};
    use crate::fee::credits::Creditable;
    use crate::fee::default_costs::EpochCosts;
    use crate::fee::default_costs::KnownCostItem::StorageDiskUsageCreditPerByte;
    use crate::query::DriveQuery;
    use crate::{common::setup_contract, drive::test_utils::TestEntropyGenerator};
    use dpp::block::epoch::Epoch;
    use dpp::data_contract::extra::common::json_document_to_document;
    use dpp::platform_value;
    use dpp::serialization_traits::PlatformSerializable;

    #[test]
    fn test_create_and_update_document_same_transaction() {
        let tmp_dir = TempDir::new().unwrap();
        let drive: Drive = Drive::open(tmp_dir, None).expect("expected to open Drive successfully");

        let db_transaction = drive.grove.start_transaction();

        drive
            .create_initial_state_structure(Some(&db_transaction))
            .expect("expected to create root tree successfully");

        let contract_cbor = hex::decode("01a5632469645820b0248cd9a27f86d05badf475dd9ff574d63219cd60c52e2be1e540c2fdd713336724736368656d61783468747470733a2f2f736368656d612e646173682e6f72672f6470702d302d342d302f6d6574612f646174612d636f6e7472616374676f776e6572496458204c9bf0db6ae315c85465e9ef26e6a006de9673731d08d14881945ddef1b5c5f26776657273696f6e0169646f63756d656e7473a267636f6e74616374a56474797065666f626a65637467696e646963657381a3646e616d656f6f6e7765724964546f55736572496466756e69717565f56a70726f7065727469657382a168246f776e6572496463617363a168746f557365724964636173636872657175697265648268746f557365724964697075626c69634b65796a70726f70657274696573a268746f557365724964a56474797065656172726179686d61784974656d731820686d696e4974656d73182069627974654172726179f570636f6e74656e744d656469615479706578216170706c69636174696f6e2f782e646173682e6470702e6964656e746966696572697075626c69634b6579a36474797065656172726179686d61784974656d73182169627974654172726179f5746164646974696f6e616c50726f70657274696573f46770726f66696c65a56474797065666f626a65637467696e646963657381a3646e616d65676f776e6572496466756e69717565f56a70726f7065727469657381a168246f776e6572496463617363687265717569726564826961766174617255726c6561626f75746a70726f70657274696573a26561626f7574a2647479706566737472696e67696d61784c656e67746818ff6961766174617255726ca3647479706566737472696e6766666f726d61746375726c696d61784c656e67746818ff746164646974696f6e616c50726f70657274696573f4").unwrap();

        drive
            .apply_contract_cbor(
                contract_cbor.clone(),
                None,
                BlockInfo::default(),
                true,
                StorageFlags::optional_default_as_cow(),
                Some(&db_transaction),
            )
            .expect("expected to apply contract successfully");

        // Create Alice profile

        let alice_profile_cbor = hex::decode("01a763246964582035edfec54aea574df968990abb47b39c206abe5c43a6157885f62958a1f1230c6524747970656770726f66696c656561626f75746a4920616d20416c69636568246f776e65724964582041d52f93f6f7c5af79ce994381c90df73cce2863d3850b9c05ef586ff0fe795f69247265766973696f6e016961766174617255726c7819687474703a2f2f746573742e636f6d2f616c6963652e6a70676f2464617461436f6e747261637449645820b0248cd9a27f86d05badf475dd9ff574d63219cd60c52e2be1e540c2fdd71333").unwrap();

        drive
            .add_cbor_serialized_document_for_serialized_contract(
                alice_profile_cbor.as_slice(),
                contract_cbor.as_slice(),
                "profile",
                None,
                true,
                BlockInfo::default(),
                true,
                StorageFlags::optional_default_as_cow(),
                Some(&db_transaction),
            )
            .expect("should create alice profile");

        // Update Alice profile

        let updated_alice_profile_cbor = hex::decode("01a763246964582035edfec54aea574df968990abb47b39c206abe5c43a6157885f62958a1f1230c6524747970656770726f66696c656561626f75746a4920616d20416c69636568246f776e65724964582041d52f93f6f7c5af79ce994381c90df73cce2863d3850b9c05ef586ff0fe795f69247265766973696f6e026961766174617255726c781a687474703a2f2f746573742e636f6d2f616c696365322e6a70676f2464617461436f6e747261637449645820b0248cd9a27f86d05badf475dd9ff574d63219cd60c52e2be1e540c2fdd71333").unwrap();

        drive
            .update_document_for_contract_cbor(
                updated_alice_profile_cbor.as_slice(),
                contract_cbor.as_slice(),
                "profile",
                None,
                BlockInfo::default(),
                true,
                StorageFlags::optional_default_as_cow(),
                Some(&db_transaction),
            )
            .expect("should update alice profile");
    }

    #[test]
    fn test_create_and_update_document_no_transactions() {
        let tmp_dir = TempDir::new().unwrap();
        let drive: Drive = Drive::open(tmp_dir, None).expect("expected to open Drive successfully");

        drive
            .create_initial_state_structure(None)
            .expect("expected to create root tree successfully");

        let contract_cbor = hex::decode("01a5632469645820b0248cd9a27f86d05badf475dd9ff574d63219cd60c52e2be1e540c2fdd713336724736368656d61783468747470733a2f2f736368656d612e646173682e6f72672f6470702d302d342d302f6d6574612f646174612d636f6e7472616374676f776e6572496458204c9bf0db6ae315c85465e9ef26e6a006de9673731d08d14881945ddef1b5c5f26776657273696f6e0169646f63756d656e7473a267636f6e74616374a56474797065666f626a65637467696e646963657381a3646e616d656f6f6e7765724964546f55736572496466756e69717565f56a70726f7065727469657382a168246f776e6572496463617363a168746f557365724964636173636872657175697265648268746f557365724964697075626c69634b65796a70726f70657274696573a268746f557365724964a56474797065656172726179686d61784974656d731820686d696e4974656d73182069627974654172726179f570636f6e74656e744d656469615479706578216170706c69636174696f6e2f782e646173682e6470702e6964656e746966696572697075626c69634b6579a36474797065656172726179686d61784974656d73182169627974654172726179f5746164646974696f6e616c50726f70657274696573f46770726f66696c65a56474797065666f626a65637467696e646963657381a3646e616d65676f776e6572496466756e69717565f56a70726f7065727469657381a168246f776e6572496463617363687265717569726564826961766174617255726c6561626f75746a70726f70657274696573a26561626f7574a2647479706566737472696e67696d61784c656e67746818ff6961766174617255726ca3647479706566737472696e6766666f726d61746375726c696d61784c656e67746818ff746164646974696f6e616c50726f70657274696573f4").unwrap();

        let contract =
            Contract::from_cbor(contract_cbor.as_slice()).expect("expected to create contract");
        drive
            .apply_contract_cbor(
                contract_cbor.clone(),
                None,
                BlockInfo::default(),
                true,
                StorageFlags::optional_default_as_cow(),
                None,
            )
            .expect("expected to apply contract successfully");

        // Create Alice profile

        let alice_profile_cbor = hex::decode("01a763246964582035edfec54aea574df968990abb47b39c206abe5c43a6157885f62958a1f1230c6524747970656770726f66696c656561626f75746a4920616d20416c69636568246f776e65724964582041d52f93f6f7c5af79ce994381c90df73cce2863d3850b9c05ef586ff0fe795f69247265766973696f6e016961766174617255726c7819687474703a2f2f746573742e636f6d2f616c6963652e6a70676f2464617461436f6e747261637449645820b0248cd9a27f86d05badf475dd9ff574d63219cd60c52e2be1e540c2fdd71333").unwrap();

        let alice_profile = Document::from_cbor(alice_profile_cbor.as_slice(), None, None)
            .expect("expected to get a document");

        let document_type = contract
            .document_type_for_name("profile")
            .expect("expected to get a document type");

        let storage_flags = Some(Cow::Owned(StorageFlags::SingleEpoch(0)));

        drive
            .add_document_for_contract(
                DocumentAndContractInfo {
                    owned_document_info: OwnedDocumentInfo {
                        document_info: DocumentRefInfo((&alice_profile, storage_flags)),
                        owner_id: None,
                    },
                    contract: &contract,
                    document_type,
                },
                true,
                BlockInfo::default(),
                true,
                None,
            )
            .expect("should create alice profile");

        let sql_string = "select * from profile";
        let query = DriveQuery::from_sql_expr(sql_string, &contract, &DriveConfig::default())
            .expect("should build query");

        let (results_no_transaction, _, _) = query
            .execute_raw_results_no_proof(&drive, None, None)
            .expect("expected to execute query");

        assert_eq!(results_no_transaction.len(), 1);

        // Update Alice profile

        let updated_alice_profile_cbor = hex::decode("01a763246964582035edfec54aea574df968990abb47b39c206abe5c43a6157885f62958a1f1230c6524747970656770726f66696c656561626f75746a4920616d20416c69636568246f776e65724964582041d52f93f6f7c5af79ce994381c90df73cce2863d3850b9c05ef586ff0fe795f69247265766973696f6e026961766174617255726c781a687474703a2f2f746573742e636f6d2f616c696365322e6a70676f2464617461436f6e747261637449645820b0248cd9a27f86d05badf475dd9ff574d63219cd60c52e2be1e540c2fdd71333").unwrap();

        drive
            .update_document_for_contract_cbor(
                updated_alice_profile_cbor.as_slice(),
                contract_cbor.as_slice(),
                "profile",
                None,
                BlockInfo::default(),
                true,
                StorageFlags::optional_default_as_cow(),
                None,
            )
            .expect("should update alice profile");

        let (results_no_transaction, _, _) = query
            .execute_raw_results_no_proof(&drive, None, None)
            .expect("expected to execute query");

        assert_eq!(results_no_transaction.len(), 1);
    }

    #[test]
    fn test_create_and_update_document_in_different_transactions() {
        let tmp_dir = TempDir::new().unwrap();
        let drive: Drive = Drive::open(tmp_dir, None).expect("expected to open Drive successfully");

        let db_transaction = drive.grove.start_transaction();

        drive
            .create_initial_state_structure(Some(&db_transaction))
            .expect("expected to create root tree successfully");

        let contract_cbor = hex::decode("01a5632469645820b0248cd9a27f86d05badf475dd9ff574d63219cd60c52e2be1e540c2fdd713336724736368656d61783468747470733a2f2f736368656d612e646173682e6f72672f6470702d302d342d302f6d6574612f646174612d636f6e7472616374676f776e6572496458204c9bf0db6ae315c85465e9ef26e6a006de9673731d08d14881945ddef1b5c5f26776657273696f6e0169646f63756d656e7473a267636f6e74616374a56474797065666f626a65637467696e646963657381a3646e616d656f6f6e7765724964546f55736572496466756e69717565f56a70726f7065727469657382a168246f776e6572496463617363a168746f557365724964636173636872657175697265648268746f557365724964697075626c69634b65796a70726f70657274696573a268746f557365724964a56474797065656172726179686d61784974656d731820686d696e4974656d73182069627974654172726179f570636f6e74656e744d656469615479706578216170706c69636174696f6e2f782e646173682e6470702e6964656e746966696572697075626c69634b6579a36474797065656172726179686d61784974656d73182169627974654172726179f5746164646974696f6e616c50726f70657274696573f46770726f66696c65a56474797065666f626a65637467696e646963657381a3646e616d65676f776e6572496466756e69717565f56a70726f7065727469657381a168246f776e6572496463617363687265717569726564826961766174617255726c6561626f75746a70726f70657274696573a26561626f7574a2647479706566737472696e67696d61784c656e67746818ff6961766174617255726ca3647479706566737472696e6766666f726d61746375726c696d61784c656e67746818ff746164646974696f6e616c50726f70657274696573f4").unwrap();

        let contract =
            Contract::from_cbor(contract_cbor.as_slice()).expect("expected to create contract");
        drive
            .apply_contract_cbor(
                contract_cbor.clone(),
                None,
                BlockInfo::default(),
                true,
                StorageFlags::optional_default_as_cow(),
                Some(&db_transaction),
            )
            .expect("expected to apply contract successfully");

        // Create Alice profile

        let alice_profile_cbor = hex::decode("01a763246964582035edfec54aea574df968990abb47b39c206abe5c43a6157885f62958a1f1230c6524747970656770726f66696c656561626f75746a4920616d20416c69636568246f776e65724964582041d52f93f6f7c5af79ce994381c90df73cce2863d3850b9c05ef586ff0fe795f69247265766973696f6e016961766174617255726c7819687474703a2f2f746573742e636f6d2f616c6963652e6a70676f2464617461436f6e747261637449645820b0248cd9a27f86d05badf475dd9ff574d63219cd60c52e2be1e540c2fdd71333").unwrap();

        let alice_profile = Document::from_cbor(alice_profile_cbor.as_slice(), None, None)
            .expect("expected to get a document");

        let document_type = contract
            .document_type_for_name("profile")
            .expect("expected to get a document type");

        let storage_flags = Some(Cow::Owned(StorageFlags::SingleEpoch(0)));

        drive
            .add_document_for_contract(
                DocumentAndContractInfo {
                    owned_document_info: OwnedDocumentInfo {
                        document_info: DocumentRefInfo((&alice_profile, storage_flags)),
                        owner_id: None,
                    },
                    contract: &contract,
                    document_type,
                },
                true,
                BlockInfo::default(),
                true,
                Some(&db_transaction),
            )
            .expect("should create alice profile");

        drive
            .grove
            .commit_transaction(db_transaction)
            .unwrap()
            .expect("should commit transaction");

        let sql_string = "select * from profile";
        let query = DriveQuery::from_sql_expr(sql_string, &contract, &DriveConfig::default())
            .expect("should build query");

        let (results_no_transaction, _, _) = query
            .execute_raw_results_no_proof(&drive, None, None)
            .expect("expected to execute query");

        assert_eq!(results_no_transaction.len(), 1);

        let db_transaction = drive.grove.start_transaction();

        let (results_on_transaction, _, _) = query
            .execute_raw_results_no_proof(&drive, None, Some(&db_transaction))
            .expect("expected to execute query");

        assert_eq!(results_on_transaction.len(), 1);

        // Update Alice profile

        let updated_alice_profile_cbor = hex::decode("01a763246964582035edfec54aea574df968990abb47b39c206abe5c43a6157885f62958a1f1230c6524747970656770726f66696c656561626f75746a4920616d20416c69636568246f776e65724964582041d52f93f6f7c5af79ce994381c90df73cce2863d3850b9c05ef586ff0fe795f69247265766973696f6e026961766174617255726c781a687474703a2f2f746573742e636f6d2f616c696365322e6a70676f2464617461436f6e747261637449645820b0248cd9a27f86d05badf475dd9ff574d63219cd60c52e2be1e540c2fdd71333").unwrap();

        drive
            .update_document_for_contract_cbor(
                updated_alice_profile_cbor.as_slice(),
                contract_cbor.as_slice(),
                "profile",
                None,
                BlockInfo::default(),
                true,
                StorageFlags::optional_default_as_cow(),
                Some(&db_transaction),
            )
            .expect("should update alice profile");

        let (results_on_transaction, _, _) = query
            .execute_raw_results_no_proof(&drive, None, Some(&db_transaction))
            .expect("expected to execute query");

        assert_eq!(results_on_transaction.len(), 1);

        drive
            .grove
            .commit_transaction(db_transaction)
            .unwrap()
            .expect("should commit transaction");
    }

    #[test]
    fn test_create_and_update_document_in_different_transactions_with_delete_rollback() {
        let tmp_dir = TempDir::new().unwrap();
        let drive: Drive = Drive::open(tmp_dir, None).expect("expected to open Drive successfully");

        let db_transaction = drive.grove.start_transaction();

        drive
            .create_initial_state_structure(Some(&db_transaction))
            .expect("expected to create root tree successfully");

        let contract_cbor = hex::decode("01a5632469645820b0248cd9a27f86d05badf475dd9ff574d63219cd60c52e2be1e540c2fdd713336724736368656d61783468747470733a2f2f736368656d612e646173682e6f72672f6470702d302d342d302f6d6574612f646174612d636f6e7472616374676f776e6572496458204c9bf0db6ae315c85465e9ef26e6a006de9673731d08d14881945ddef1b5c5f26776657273696f6e0169646f63756d656e7473a267636f6e74616374a56474797065666f626a65637467696e646963657381a3646e616d656f6f6e7765724964546f55736572496466756e69717565f56a70726f7065727469657382a168246f776e6572496463617363a168746f557365724964636173636872657175697265648268746f557365724964697075626c69634b65796a70726f70657274696573a268746f557365724964a56474797065656172726179686d61784974656d731820686d696e4974656d73182069627974654172726179f570636f6e74656e744d656469615479706578216170706c69636174696f6e2f782e646173682e6470702e6964656e746966696572697075626c69634b6579a36474797065656172726179686d61784974656d73182169627974654172726179f5746164646974696f6e616c50726f70657274696573f46770726f66696c65a56474797065666f626a65637467696e646963657381a3646e616d65676f776e6572496466756e69717565f56a70726f7065727469657381a168246f776e6572496463617363687265717569726564826961766174617255726c6561626f75746a70726f70657274696573a26561626f7574a2647479706566737472696e67696d61784c656e67746818ff6961766174617255726ca3647479706566737472696e6766666f726d61746375726c696d61784c656e67746818ff746164646974696f6e616c50726f70657274696573f4").unwrap();

        let contract =
            Contract::from_cbor(contract_cbor.as_slice()).expect("expected to create contract");
        drive
            .apply_contract_cbor(
                contract_cbor.clone(),
                None,
                BlockInfo::default(),
                true,
                StorageFlags::optional_default_as_cow(),
                Some(&db_transaction),
            )
            .expect("expected to apply contract successfully");

        // Create Alice profile

        let alice_profile_cbor = hex::decode("01a763246964582035edfec54aea574df968990abb47b39c206abe5c43a6157885f62958a1f1230c6524747970656770726f66696c656561626f75746a4920616d20416c69636568246f776e65724964582041d52f93f6f7c5af79ce994381c90df73cce2863d3850b9c05ef586ff0fe795f69247265766973696f6e016961766174617255726c7819687474703a2f2f746573742e636f6d2f616c6963652e6a70676f2464617461436f6e747261637449645820b0248cd9a27f86d05badf475dd9ff574d63219cd60c52e2be1e540c2fdd71333").unwrap();

        let alice_profile = Document::from_cbor(alice_profile_cbor.as_slice(), None, None)
            .expect("expected to get a document");

        let document_type = contract
            .document_type_for_name("profile")
            .expect("expected to get a document type");

        let storage_flags = Some(Cow::Owned(StorageFlags::SingleEpoch(0)));

        drive
            .add_document_for_contract(
                DocumentAndContractInfo {
                    owned_document_info: OwnedDocumentInfo {
                        document_info: DocumentRefInfo((&alice_profile, storage_flags)),
                        owner_id: None,
                    },
                    contract: &contract,
                    document_type,
                },
                true,
                BlockInfo::default(),
                true,
                Some(&db_transaction),
            )
            .expect("should create alice profile");

        drive
            .grove
            .commit_transaction(db_transaction)
            .unwrap()
            .expect("should commit transaction");

        let sql_string = "select * from profile";
        let query = DriveQuery::from_sql_expr(sql_string, &contract, &DriveConfig::default())
            .expect("should build query");

        let (results_no_transaction, _, _) = query
            .execute_raw_results_no_proof(&drive, None, None)
            .expect("expected to execute query");

        assert_eq!(results_no_transaction.len(), 1);

        let db_transaction = drive.grove.start_transaction();

        let (results_on_transaction, _, _) = query
            .execute_raw_results_no_proof(&drive, None, Some(&db_transaction))
            .expect("expected to execute query");

        assert_eq!(results_on_transaction.len(), 1);

        drive
            .delete_document_for_contract(
                alice_profile.id.to_buffer(),
                &contract,
                "profile",
                None,
                BlockInfo::default(),
                true,
                Some(&db_transaction),
            )
            .expect("expected to delete document");

        let (results_on_transaction, _, _) = query
            .execute_raw_results_no_proof(&drive, None, Some(&db_transaction))
            .expect("expected to execute query");

        assert_eq!(results_on_transaction.len(), 0);

        drive
            .grove
            .rollback_transaction(&db_transaction)
            .expect("expected to rollback transaction");

        let (results_on_transaction, _, _) = query
            .execute_raw_results_no_proof(&drive, None, Some(&db_transaction))
            .expect("expected to execute query");

        assert_eq!(results_on_transaction.len(), 1);

        // Update Alice profile

        let updated_alice_profile_cbor = hex::decode("01a763246964582035edfec54aea574df968990abb47b39c206abe5c43a6157885f62958a1f1230c6524747970656770726f66696c656561626f75746a4920616d20416c69636568246f776e65724964582041d52f93f6f7c5af79ce994381c90df73cce2863d3850b9c05ef586ff0fe795f69247265766973696f6e026961766174617255726c781a687474703a2f2f746573742e636f6d2f616c696365322e6a70676f2464617461436f6e747261637449645820b0248cd9a27f86d05badf475dd9ff574d63219cd60c52e2be1e540c2fdd71333").unwrap();

        drive
            .update_document_for_contract_cbor(
                updated_alice_profile_cbor.as_slice(),
                contract_cbor.as_slice(),
                "profile",
                None,
                BlockInfo::default(),
                true,
                StorageFlags::optional_default_as_cow(),
                Some(&db_transaction),
            )
            .expect("should update alice profile");
    }

    #[test]
    fn test_create_update_and_delete_document() {
        let tmp_dir = TempDir::new().unwrap();
        let drive: Drive = Drive::open(tmp_dir, None).expect("expected to open Drive successfully");

        drive
            .create_initial_state_structure(None)
            .expect("should create root tree");

        let contract = json!({
            "protocolVersion": 1,
            "$id": "BZUodcFoFL6KvnonehrnMVggTvCe8W5MiRnZuqLb6M54",
            "$schema": "https://schema.dash.org/dpp-0-4-0/meta/data-contract",
            "version": 1,
            "ownerId": "GZVdTnLFAN2yE9rLeCHBDBCr7YQgmXJuoExkY347j7Z5",
            "documents": {
                "indexedDocument": {
                    "type": "object",
                    "indices": [
                        {"name":"index1", "properties": [{"$ownerId":"asc"}, {"firstName":"desc"}], "unique":true},
                        {"name":"index2", "properties": [{"$ownerId":"asc"}, {"lastName":"desc"}], "unique":true},
                        {"name":"index3", "properties": [{"lastName":"asc"}]},
                        {"name":"index4", "properties": [{"$createdAt":"asc"}, {"$updatedAt":"asc"}]},
                        {"name":"index5", "properties": [{"$updatedAt":"asc"}]},
                        {"name":"index6", "properties": [{"$createdAt":"asc"}]}
                    ],
                    "properties":{
                        "firstName": {
                            "type": "string",
                            "maxLength": 63,
                        },
                        "lastName": {
                            "type": "string",
                            "maxLength": 63,
                        }
                    },
                    "required": ["firstName", "$createdAt", "$updatedAt", "lastName"],
                    "additionalProperties": false,
                },
            },
        });

        let contract = cbor_serializer::serializable_value_to_cbor(
            &contract,
            Some(defaults::PROTOCOL_VERSION),
        )
        .expect("expected to serialize to cbor");

        drive
            .apply_contract_cbor(
                contract.clone(),
                None,
                BlockInfo::default(),
                true,
                StorageFlags::optional_default_as_cow(),
                None,
            )
            .expect("should create a contract");

        // Create document

        let document = json!({
           "$protocolVersion": 1,
           "$id": "DLRWw2eRbLAW5zDU2c7wwsSFQypTSZPhFYzpY48tnaXN",
           "$type": "indexedDocument",
           "$dataContractId": "BZUodcFoFL6KvnonehrnMVggTvCe8W5MiRnZuqLb6M54",
           "$ownerId": "GZVdTnLFAN2yE9rLeCHBDBCr7YQgmXJuoExkY347j7Z5",
           "$revision": 1,
           "firstName": "myName",
           "lastName": "lastName",
           "$createdAt":1647535750329_u64,
           "$updatedAt":1647535750329_u64,
        });

        let serialized_document = cbor_serializer::serializable_value_to_cbor(
            &document,
            Some(defaults::PROTOCOL_VERSION),
        )
        .expect("expected to serialize to cbor");

        drive
            .add_cbor_serialized_document_for_serialized_contract(
                serialized_document.as_slice(),
                contract.as_slice(),
                "indexedDocument",
                None,
                true,
                BlockInfo::default(),
                true,
                StorageFlags::optional_default_as_cow(),
                None,
            )
            .expect("should add document");

        // Update document

        let document = json!({
           "$protocolVersion": 1,
           "$id": "DLRWw2eRbLAW5zDU2c7wwsSFQypTSZPhFYzpY48tnaXN",
           "$type": "indexedDocument",
           "$dataContractId": "BZUodcFoFL6KvnonehrnMVggTvCe8W5MiRnZuqLb6M54",
           "$ownerId": "GZVdTnLFAN2yE9rLeCHBDBCr7YQgmXJuoExkY347j7Z5",
           "$revision": 2,
           "firstName": "updatedName",
           "lastName": "lastName",
           "$createdAt":1647535750329_u64,
           "$updatedAt":1647535754556_u64,
        });

        let serialized_document = cbor_serializer::serializable_value_to_cbor(
            &document,
            Some(defaults::PROTOCOL_VERSION),
        )
        .expect("expected to serialize to cbor");

        drive
            .update_document_for_contract_cbor(
                serialized_document.as_slice(),
                contract.as_slice(),
                "indexedDocument",
                None,
                BlockInfo::default(),
                true,
                StorageFlags::optional_default_as_cow(),
                None,
            )
            .expect("should update document");

        let document_id = bs58::decode("DLRWw2eRbLAW5zDU2c7wwsSFQypTSZPhFYzpY48tnaXN")
            .into_vec()
            .expect("should decode")
            .as_slice()
            .try_into()
            .expect("this be 32 bytes");

        // Delete document

        drive
            .delete_document_for_contract_cbor(
                document_id,
                &contract,
                "indexedDocument",
                None,
                BlockInfo::default(),
                true,
                None,
            )
            .expect("should delete document");
    }

    #[test]
    fn test_modify_dashpay_contact_request() {
        let tmp_dir = TempDir::new().unwrap();
        let drive: Drive = Drive::open(tmp_dir, None).expect("expected to open Drive successfully");

        let db_transaction = drive.grove.start_transaction();

        drive
            .create_initial_state_structure(Some(&db_transaction))
            .expect("expected to create root tree successfully");

        let contract = setup_contract(
            &drive,
            "tests/supporting_files/contract/dashpay/dashpay-contract.json",
            None,
            Some(&db_transaction),
        );

        let document_type = contract
            .document_type_for_name("contactRequest")
            .expect("expected to get document type");

        let random_owner_id = rand::thread_rng().gen::<[u8; 32]>();

        let dashpay_cr_document = json_document_to_document(
            "tests/supporting_files/contract/dashpay/contact-request0.json",
            Some(random_owner_id.into()),
            document_type,
        )
        .expect("expected to get document");

        drive
            .add_document_for_contract(
                DocumentAndContractInfo {
                    owned_document_info: OwnedDocumentInfo {
                        document_info: DocumentRefInfo((
                            &dashpay_cr_document,
                            StorageFlags::optional_default_as_cow(),
                        )),
                        owner_id: Some(random_owner_id),
                    },
                    contract: &contract,
                    document_type,
                },
                false,
                BlockInfo::default(),
                true,
                Some(&db_transaction),
            )
            .expect("expected to insert a document successfully");

        drive
            .update_document_for_contract(
                &dashpay_cr_document,
                &contract,
                document_type,
                Some(random_owner_id),
                BlockInfo::default(),
                true,
                StorageFlags::optional_default_as_cow(),
                Some(&db_transaction),
            )
            .expect_err("expected not to be able to update a non mutable document");

        drive
            .add_document_for_contract(
                DocumentAndContractInfo {
                    owned_document_info: OwnedDocumentInfo {
                        document_info: DocumentRefInfo((
                            &dashpay_cr_document,
                            StorageFlags::optional_default_as_cow(),
                        )),
                        owner_id: Some(random_owner_id),
                    },
                    contract: &contract,
                    document_type,
                },
                true,
                BlockInfo::default(),
                true,
                Some(&db_transaction),
            )
            .expect_err("expected not to be able to override a non mutable document");
    }

    #[test]
    fn test_update_dashpay_profile_with_history() {
        let tmp_dir = TempDir::new().unwrap();
        let drive: Drive = Drive::open(tmp_dir, None).expect("expected to open Drive successfully");

        let db_transaction = drive.grove.start_transaction();

        drive
            .create_initial_state_structure(Some(&db_transaction))
            .expect("expected to create root tree successfully");

        let contract = setup_contract(
            &drive,
            "tests/supporting_files/contract/dashpay/dashpay-contract-with-profile-history.json",
            None,
            Some(&db_transaction),
        );

        let document_type = contract
            .document_type_for_name("profile")
            .expect("expected to get document type");

        let random_owner_id = rand::thread_rng().gen::<[u8; 32]>();

        let dashpay_profile_document = json_document_to_document(
            "tests/supporting_files/contract/dashpay/profile0.json",
            Some(random_owner_id.into()),
            document_type,
        )
        .expect("expected to get cbor document");

        let dashpay_profile_updated_public_message_document = json_document_to_document(
            "tests/supporting_files/contract/dashpay/profile0-updated-public-message.json",
            Some(random_owner_id.into()),
            document_type,
        )
        .expect("expected to get cbor document");

        drive
            .add_document_for_contract(
                DocumentAndContractInfo {
                    owned_document_info: OwnedDocumentInfo {
                        document_info: DocumentRefInfo((
                            &dashpay_profile_document,
                            StorageFlags::optional_default_as_cow(),
                        )),
                        owner_id: None,
                    },
                    contract: &contract,
                    document_type,
                },
                false,
                BlockInfo::default(),
                true,
                Some(&db_transaction),
            )
            .expect("expected to insert a document successfully");

        drive
            .update_document_for_contract(
                &dashpay_profile_updated_public_message_document,
                &contract,
                document_type,
                Some(random_owner_id),
                BlockInfo::default(),
                true,
                StorageFlags::optional_default_as_cow(),
                Some(&db_transaction),
            )
            .expect("expected to update a document with history successfully");
    }

    fn test_fees_for_update_document(using_history: bool, using_transaction: bool) {
        let config = DriveConfig {
            batching_consistency_verification: true,
            has_raw_enabled: true,
            default_genesis_time: Some(0),
            ..Default::default()
        };
        let tmp_dir = TempDir::new().unwrap();

        let drive: Drive =
            Drive::open(&tmp_dir, Some(config)).expect("expected to open Drive successfully");

        let transaction = if using_transaction {
            Some(drive.grove.start_transaction())
        } else {
            None
        };

        drive
            .create_initial_state_structure(transaction.as_ref())
            .expect("expected to create root tree successfully");

        let path = if using_history {
            "tests/supporting_files/contract/family/family-contract-with-history-only-message-index.json"
        } else {
            "tests/supporting_files/contract/family/family-contract-only-message-index.json"
        };

        // setup code
        let contract = setup_contract(&drive, path, None, transaction.as_ref());

        let id = Identifier::from([1u8; 32]);
        let owner_id = Identifier::from([2u8; 32]);
        let person_0_original = Person {
            id,
            owner_id,
            first_name: "Samuel".to_string(),
            middle_name: "Abraham".to_string(),
            last_name: "Westrich".to_string(),
            message: Some("My apples are safe".to_string()),
            age: 33,
        };

        let person_0_updated = Person {
            id,
            owner_id,
            first_name: "Samuel".to_string(),
            middle_name: "Abraham".to_string(),
            last_name: "Westrich2".to_string(),
            message: Some("My apples are safe".to_string()),
            age: 35,
        };

        let document_type = contract
            .document_type_for_name("person")
            .expect("expected to get document type");

        let value = platform_value::to_value(&person_0_original).expect("person into value");

        let document: Document = platform_value::from_value(value).expect("value to document");

        let document_serialized = document
            .serialize_consume(document_type)
            .expect("expected to serialize document");

        assert_eq!(document_serialized.len(), 115);
        let original_fees = apply_person(
            &drive,
            &contract,
            BlockInfo::default(),
            &person_0_original,
            true,
            transaction.as_ref(),
        );
        let original_bytes = original_fees.storage_fee
            / Epoch::new(0)
                .unwrap()
                .cost_for_known_cost_item(StorageDiskUsageCreditPerByte);
        let expected_added_bytes = if using_history {
            //Explanation for 1235

            //todo
            1235
        } else {
            //Explanation for 959

            // Document Storage

            //// Item
            // = 356 Bytes

            // Explanation for 354 storage_written_bytes

            // Key -> 65 bytes
            // 32 bytes for the key prefix
            // 32 bytes for the unique id
            // 1 byte for key_size (required space for 64)

            // Value -> 221
            //   1 for the flag option with flags
            //   1 for the flags size
            //   35 for flags 32 + 1 + 2
            //   1 for the enum type
            //   1 for item
            //   117 for item serialized bytes (verified above)
            //   1 for Basic Merk
            // 32 for node hash
            // 32 for value hash
            // 2 byte for the value_size (required space for above 128)

            // Parent Hook -> 68
            // Key Bytes 32
            // Hash Size 32
            // Key Length 1
            // Child Heights 2
            // Basic Merk 1

            // Total 65 + 221 + 68 = 354

            //// Tree 1 / <Person Contract> / 1 / person / message
            // Key: My apples are safe
            // = 179 Bytes

            // Explanation for 179 storage_written_bytes

            // Key -> 51 bytes
            // 32 bytes for the key prefix
            // 18 bytes for the key "My apples are safe" 18 characters
            // 1 byte for key_size (required space for 50)

            // Value -> 74
            //   1 for the flag option with flags
            //   1 for the flags size
            //   35 for flags
            //   1 for the enum type
            //   1 for empty tree value
            //   1 for Basic Merk
            // 32 for node hash
            // 0 for value hash
            // 2 byte for the value_size (required space for 73 + up to 256 for child key)

            // Parent Hook -> 54
            // Key Bytes 18
            // Hash Size 32
            // Key Length 1
            // Child Heights 2
            // Basic merk 1

            // Total 51 + 74 + 54 = 179

            //// Tree 1 / <Person Contract> / 1 / person / message / My apples are safe
            // Key: 0
            // = 145 Bytes

            // Explanation for 145 storage_written_bytes

            // Key -> 34 bytes
            // 32 bytes for the key prefix
            // 1 bytes for the key "My apples are safe" 18 characters
            // 1 byte for key_size (required space for 33)

            // Value -> 74
            //   1 for the flag option with flags
            //   1 for the flags size
            //   35 for flags
            //   1 for the enum type
            //   1 for empty tree value
            // 32 for node hash
            // 0 for value hash
            // 1 for Basic Merk
            // 2 byte for the value_size (required space for 73 + up to 256 for child key)

            // Parent Hook -> 37
            // Key Bytes 1
            // Hash Size 32
            // Key Length 1
            // Child Heights 2
            // Basic Merk 1

            // Total 34 + 74 + 37 = 145

            //// Ref 1 / <Person Contract> / 1 / person / message / My apples are safe
            // Reference to Serialized Item
            // = 276 Bytes

            // Explanation for 276 storage_written_bytes

            // Key -> 65 bytes
            // 32 bytes for the key prefix
            // 32 bytes for the unique id
            // 1 byte for key_size (required space for 64)

            // Value -> 145
            //   1 for the flag option with flags
            //   1 for the flags size
            //   35 for flags 32 + 1 + 2
            //   1 for the element type as reference
            //   1 for reference type as upstream root reference
            //   1 for reference root height
            //   36 for the reference path bytes ( 1 + 1 + 32 + 1 + 1)
            //   2 for the max reference hop
            // 32 for node hash
            // 32 for value hash
            // 1 for Basic Merk
            // 2 byte for the value_size (required space for above 128)

            // Parent Hook -> 68
            // Key Bytes 32
            // Hash Size 32
            // Key Length 1
            // Child Heights 2
            // Basic Merk 1

            // Total 65 + 145 + 68 = 278

            //// 356 + 179 + 145 + 278

            959
        };
        assert_eq!(original_bytes, expected_added_bytes);

        if !using_history {
            // let's delete it, just to make sure everything is working.
            // we can delete items that use history though
            let deletion_fees = delete_person(
                &drive,
                &contract,
                BlockInfo::default(),
                &person_0_original,
                transaction.as_ref(),
            );

            let removed_credits = deletion_fees
                .fee_refunds
                .get(owner_id.as_bytes())
                .unwrap()
                .get(&0)
                .unwrap();

            assert_eq!(*removed_credits, 25827688);
            let refund_equivalent_bytes = removed_credits.to_unsigned()
                / Epoch::new(0)
                    .unwrap()
                    .cost_for_known_cost_item(StorageDiskUsageCreditPerByte);

            assert!(expected_added_bytes > refund_equivalent_bytes);
            assert_eq!(refund_equivalent_bytes, 956); // we refunded 956 instead of 959

            // let's re-add it again
            let original_fees = apply_person(
                &drive,
                &contract,
                BlockInfo::default(),
                &person_0_original,
                true,
                transaction.as_ref(),
            );

            let original_bytes = original_fees.storage_fee
                / Epoch::new(0)
                    .unwrap()
                    .cost_for_known_cost_item(StorageDiskUsageCreditPerByte);

            assert_eq!(original_bytes, expected_added_bytes);
        }

        // now let's update it 1 second later
        let update_fees = apply_person(
            &drive,
            &contract,
            BlockInfo::default_with_time(1000),
            &person_0_updated,
            true,
            transaction.as_ref(),
        );
        // we both add and remove bytes
        // this is because trees are added because of indexes, and also removed
        let added_bytes = update_fees.storage_fee
            / Epoch::new(0)
                .unwrap()
                .cost_for_known_cost_item(StorageDiskUsageCreditPerByte);

        let expected_added_bytes = if using_history { 310 } else { 1 };
        assert_eq!(added_bytes, expected_added_bytes);
    }

    fn test_fees_for_update_document_on_index(using_history: bool, using_transaction: bool) {
        let config = DriveConfig {
            batching_consistency_verification: true,
            has_raw_enabled: true,
            default_genesis_time: Some(0),
            ..Default::default()
        };
        let tmp_dir = TempDir::new().unwrap();

        let drive: Drive =
            Drive::open(&tmp_dir, Some(config)).expect("expected to open Drive successfully");

        let transaction = if using_transaction {
            Some(drive.grove.start_transaction())
        } else {
            None
        };

        drive
            .create_initial_state_structure(transaction.as_ref())
            .expect("expected to create root tree successfully");

        let path = if using_history {
            "tests/supporting_files/contract/family/family-contract-with-history-only-message-index.json"
        } else {
            "tests/supporting_files/contract/family/family-contract-only-message-index.json"
        };

        // setup code
        let contract = setup_contract(&drive, path, None, transaction.as_ref());

        let id = Identifier::from([1u8; 32]);
        let owner_id = Identifier::from([2u8; 32]);
        let person_0_original = Person {
            id,
            owner_id,
            first_name: "Samuel".to_string(),
            middle_name: "Abraham".to_string(),
            last_name: "Westrich".to_string(),
            message: Some("My apples are safe".to_string()),
            age: 33,
        };

        let person_0_updated = Person {
            id,
            owner_id,
            first_name: "Samuel".to_string(),
            middle_name: "Abraham".to_string(),
            last_name: "Westrich".to_string(),
            message: Some("My apples are safer".to_string()),
            age: 35,
        };

        let original_fees = apply_person(
            &drive,
            &contract,
            BlockInfo::default(),
            &person_0_original,
            true,
            transaction.as_ref(),
        );
        let original_bytes = original_fees.storage_fee
            / Epoch::new(0)
                .unwrap()
                .cost_for_known_cost_item(StorageDiskUsageCreditPerByte);
        let expected_added_bytes = if using_history { 1235 } else { 959 };
        assert_eq!(original_bytes, expected_added_bytes);
        if !using_history {
            // let's delete it, just to make sure everything is working.
            let deletion_fees = delete_person(
                &drive,
                &contract,
                BlockInfo::default(),
                &person_0_original,
                transaction.as_ref(),
            );

            let removed_credits = deletion_fees
                .fee_refunds
                .get(owner_id.as_bytes())
                .unwrap()
                .get(&0)
                .unwrap();

            assert_eq!(*removed_credits, 25827688);
            let refund_equivalent_bytes = removed_credits.to_unsigned()
                / Epoch::new(0)
                    .unwrap()
                    .cost_for_known_cost_item(StorageDiskUsageCreditPerByte);

            assert!(expected_added_bytes > refund_equivalent_bytes);
            assert_eq!(refund_equivalent_bytes, 956); // we refunded 1008 instead of 1011

            // let's re-add it again
            let original_fees = apply_person(
                &drive,
                &contract,
                BlockInfo::default(),
                &person_0_original,
                true,
                transaction.as_ref(),
            );

            let original_bytes = original_fees.storage_fee
                / Epoch::new(0)
                    .unwrap()
                    .cost_for_known_cost_item(StorageDiskUsageCreditPerByte);

            assert_eq!(original_bytes, expected_added_bytes);
        }

        // now let's update it
        let update_fees = apply_person(
            &drive,
            &contract,
            BlockInfo::default(),
            &person_0_updated,
            true,
            transaction.as_ref(),
        );
        // we both add and remove bytes
        // this is because trees are added because of indexes, and also removed
        let added_bytes = update_fees.storage_fee
            / Epoch::new(0)
                .unwrap()
                .cost_for_known_cost_item(StorageDiskUsageCreditPerByte);

        let removed_credits = update_fees
            .fee_refunds
            .get(owner_id.as_bytes())
            .unwrap()
            .get(&0)
            .unwrap();

        // We added one byte, and since it is an index, and keys are doubled it's 2 extra bytes
        let expected_added_bytes = if using_history { 607 } else { 605 };
        assert_eq!(added_bytes, expected_added_bytes);

        let expected_removed_credits = if using_history { 16266750 } else { 16212825 };
        assert_eq!(*removed_credits, expected_removed_credits);
        let refund_equivalent_bytes = removed_credits.to_unsigned()
            / Epoch::new(0)
                .unwrap()
                .cost_for_known_cost_item(StorageDiskUsageCreditPerByte);

        assert!(expected_added_bytes > refund_equivalent_bytes);
        let expected_remove_bytes = if using_history { 602 } else { 600 };
        assert_eq!(refund_equivalent_bytes, expected_remove_bytes); // we refunded 1011 instead of 1014
    }

    #[test]
    fn test_fees_for_update_document_no_history_using_transaction() {
        test_fees_for_update_document(false, true)
    }

    #[test]
    fn test_fees_for_update_document_no_history_no_transaction() {
        test_fees_for_update_document(false, false)
    }

    #[test]
    fn test_fees_for_update_document_with_history_using_transaction() {
        test_fees_for_update_document(true, true)
    }

    #[test]
    fn test_fees_for_update_document_with_history_no_transaction() {
        test_fees_for_update_document(true, false)
    }

    #[test]
    fn test_fees_for_update_document_on_index_no_history_using_transaction() {
        test_fees_for_update_document_on_index(false, true)
    }

    #[test]
    fn test_fees_for_update_document_on_index_no_history_no_transaction() {
        test_fees_for_update_document_on_index(false, false)
    }

    #[test]
    fn test_fees_for_update_document_on_index_with_history_using_transaction() {
        test_fees_for_update_document_on_index(true, true)
    }

    #[test]
    fn test_fees_for_update_document_on_index_with_history_no_transaction() {
        test_fees_for_update_document_on_index(true, false)
    }

    fn test_estimated_fees_for_update_document(using_history: bool, using_transaction: bool) {
        let config = DriveConfig {
            batching_consistency_verification: true,
            has_raw_enabled: true,
            default_genesis_time: Some(0),
            ..Default::default()
        };
        let tmp_dir = TempDir::new().unwrap();

        let drive: Drive =
            Drive::open(&tmp_dir, Some(config)).expect("expected to open Drive successfully");

        let transaction = if using_transaction {
            Some(drive.grove.start_transaction())
        } else {
            None
        };

        drive
            .create_initial_state_structure(transaction.as_ref())
            .expect("expected to create root tree successfully");

        let path = if using_history {
            "tests/supporting_files/contract/family/family-contract-with-history-only-message-index.json"
        } else {
            "tests/supporting_files/contract/family/family-contract-only-message-index.json"
        };

        // setup code
        let contract = setup_contract(&drive, path, None, transaction.as_ref());

        let id = Identifier::from([1u8; 32]);
        let owner_id = Identifier::from([2u8; 32]);
        let person_0_original = Person {
            id,
            owner_id,
            first_name: "Samuel".to_string(),
            middle_name: "Abraham".to_string(),
            last_name: "Westrich".to_string(),
            message: Some("My apples are safe".to_string()),
            age: 33,
        };

        let person_0_updated = Person {
            id,
            owner_id,
            first_name: "Samuel".to_string(),
            middle_name: "Abraham".to_string(),
            last_name: "Westrich2".to_string(),
            message: Some("My apples are safe".to_string()),
            age: 35,
        };

        let original_fees = apply_person(
            &drive,
            &contract,
            BlockInfo::default(),
            &person_0_original,
            false,
            transaction.as_ref(),
        );
        let original_bytes = original_fees.storage_fee
            / Epoch::new(0)
                .unwrap()
                .cost_for_known_cost_item(StorageDiskUsageCreditPerByte);
        let expected_added_bytes = if using_history {
            //Explanation for 1235

            //todo
            1235
        } else {
            //Explanation for 959

            // Document Storage

            //// Item
            // = 355 Bytes

            // Explanation for 355 storage_written_bytes

            // Key -> 65 bytes
            // 32 bytes for the key prefix
            // 32 bytes for the unique id
            // 1 byte for key_size (required space for 64)

            // Value -> 222
            //   1 for the flag option with flags
            //   1 for the flags size
            //   35 for flags 32 + 1 + 2
            //   1 for the enum type
            //   1 for item
            //   116 for item serialized bytes
            //   1 for Basic Merk
            // 32 for node hash
            // 32 for value hash
            // 2 byte for the value_size (required space for above 128)

            // Parent Hook -> 68
            // Key Bytes 32
            // Hash Size 32
            // Key Length 1
            // Child Heights 2
            // Feature Type Basic 1

            // Total 65 + 222 + 68 = 355

            //// Tree 1 / <Person Contract> / 1 / person / message
            // Key: My apples are safe
            // = 177 Bytes

            // Explanation for 177 storage_written_bytes

            // Key -> 51 bytes
            // 32 bytes for the key prefix
            // 18 bytes for the key "My apples are safe" 18 characters
            // 1 byte for key_size (required space for 50)

            // Value -> 74
            //   1 for the flag option with flags
            //   1 for the flags size
            //   35 for flags
            //   1 for the enum type
            //   1 for empty tree value
            //   1 for Basic Merk
            // 32 for node hash
            // 0 for value hash
            // 2 byte for the value_size (required space for 73 + up to 256 for child key)

            // Parent Hook -> 54
            // Key Bytes 18
            // Hash Size 32
            // Key Length 1
            // Child Heights 2
            // Basic Merk 1

            // Total 51 + 74 + 54 = 179

            //// Tree 1 / <Person Contract> / 1 / person / message / My apples are safe
            // Key: 0
            // = 143 Bytes

            // Explanation for 145 storage_written_bytes

            // Key -> 34 bytes
            // 32 bytes for the key prefix
            // 1 bytes for the key "My apples are safe" 18 characters
            // 1 byte for key_size (required space for 33)

            // Value -> 74
            //   1 for the flag option with flags
            //   1 for the flags size
            //   35 for flags
            //   1 for the enum type
            //   1 for empty tree value
            //   1 for Basic Merk
            // 32 for node hash
            // 0 for value hash
            // 2 byte for the value_size (required space for 73 + up to 256 for child key)

            // Parent Hook -> 37
            // Key Bytes 1
            // Hash Size 32
            // Key Length 1
            // Child Heights 2
            // Basic Merk 1

            // Total 34 + 74 + 37 = 145

            //// Ref 1 / <Person Contract> / 1 / person / message / My apples are safe
            // Reference to Serialized Item
            // = 319 Bytes

            // Explanation for 276 storage_written_bytes

            // Key -> 65 bytes
            // 32 bytes for the key prefix
            // 32 bytes for the unique id
            // 1 byte for key_size (required space for 64)

            // Value -> 145
            //   1 for the flag option with flags
            //   1 for the flags size
            //   35 for flags 32 + 1 + 2
            //   1 for the element type as reference
            //   1 for reference type as upstream root reference
            //   1 for reference root height
            //   36 for the reference path bytes ( 1 + 1 + 32 + 1 + 1)
            //   2 for the max reference hop
            //   1 for Basic Merk
            // 32 for node hash
            // 32 for value hash
            // 2 byte for the value_size (required space for above 128)

            // Parent Hook -> 68
            // Key Bytes 32
            // Hash Size 32
            // Key Length 1
            // Child Heights 2
            // No Sum Tree 1

            // Total 65 + 145 + 68 = 278

            // 357 + 179 + 145 + 278 = 959

            959
        };
        assert_eq!(original_bytes, expected_added_bytes);

        // now let's update it 1 second later
        let update_fees = apply_person(
            &drive,
            &contract,
            BlockInfo::default_with_time(1000),
            &person_0_updated,
            false,
            transaction.as_ref(),
        );
        // we both add and remove bytes
        // this is because trees are added because of indexes, and also removed
        let added_bytes = update_fees.storage_fee
            / Epoch::new(0)
                .unwrap()
                .cost_for_known_cost_item(StorageDiskUsageCreditPerByte);

        let expected_added_bytes = if using_history { 1236 } else { 960 };
        assert_eq!(added_bytes, expected_added_bytes);
    }

    #[test]
    fn test_estimated_fees_for_update_document_no_history_using_transaction() {
        test_estimated_fees_for_update_document(false, true)
    }

    #[test]
    fn test_estimated_fees_for_update_document_no_history_no_transaction() {
        test_estimated_fees_for_update_document(false, false)
    }

    #[test]
    fn test_estimated_fees_for_update_document_with_history_using_transaction() {
        test_estimated_fees_for_update_document(true, true)
    }

    #[test]
    fn test_estimated_fees_for_update_document_with_history_no_transaction() {
        test_estimated_fees_for_update_document(true, false)
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Person {
        #[serde(rename = "$id")]
        id: Identifier,
        #[serde(rename = "$ownerId")]
        owner_id: Identifier,
        first_name: String,
        middle_name: String,
        last_name: String,
        message: Option<String>,
        age: u8,
    }

    fn apply_person(
        drive: &Drive,
        contract: &Contract,
        block_info: BlockInfo,
        person: &Person,
        apply: bool,
        transaction: TransactionArg,
    ) -> FeeResult {
        let document_type = contract
            .document_type_for_name("person")
            .expect("expected to get document type");

        let value = platform_value::to_value(person).expect("person into value");

        let document = platform_value::from_value(value).expect("value to document");

        let storage_flags = Some(Cow::Owned(StorageFlags::SingleEpochOwned(
            0,
            person.owner_id.to_buffer(),
        )));

        drive
            .add_document_for_contract(
                DocumentAndContractInfo {
                    owned_document_info: OwnedDocumentInfo {
                        document_info: DocumentRefInfo((&document, storage_flags)),
                        owner_id: None,
                    },
                    contract,
                    document_type,
                },
                true,
                block_info,
                apply,
                transaction,
            )
            .expect("expected to add document")
    }

    fn delete_person(
        drive: &Drive,
        contract: &Contract,
        block_info: BlockInfo,
        person: &Person,
        transaction: TransactionArg,
    ) -> FeeResult {
        drive
            .delete_document_for_contract(
                person.id.to_buffer(),
                contract,
                "person",
                Some(person.owner_id.to_buffer()),
                block_info,
                true,
                transaction,
            )
            .expect("expected to remove person")
    }

    fn test_update_complex_person(
        using_history: bool,
        using_transaction: bool,
        using_has_raw: bool,
    ) {
        let config = DriveConfig {
            batching_consistency_verification: true,
            has_raw_enabled: using_has_raw,
            default_genesis_time: Some(0),
            ..Default::default()
        };
        let tmp_dir = TempDir::new().unwrap();

        let drive: Drive =
            Drive::open(&tmp_dir, Some(config)).expect("expected to open Drive successfully");

        let transaction = if using_transaction {
            Some(drive.grove.start_transaction())
        } else {
            None
        };

        drive
            .create_initial_state_structure(transaction.as_ref())
            .expect("expected to create root tree successfully");

        let path = if using_history {
            "tests/supporting_files/contract/family/family-contract-with-history-only-message-index.json"
        } else {
            "tests/supporting_files/contract/family/family-contract-only-message-index.json"
        };

        // setup code
        let contract = setup_contract(&drive, path, None, transaction.as_ref());

        let person_0_original = Person {
            id: Identifier::from([0u8; 32]),
            owner_id: Identifier::from([0u8; 32]),
            first_name: "Samuel".to_string(),
            middle_name: "Abraham".to_string(),
            last_name: "Westrich".to_string(),
            message: Some("My apples are safe".to_string()),
            age: 33,
        };

        let person_0_updated = Person {
            id: Identifier::from([0u8; 32]),
            owner_id: Identifier::from([0u8; 32]),
            first_name: "Samuel".to_string(),
            middle_name: "Abraham".to_string(),
            last_name: "Westrich".to_string(),
            message: Some("Lemons are now my thing too".to_string()),
            age: 35,
        };

        let person_1_original = Person {
            id: Identifier::from([1u8; 32]),
            owner_id: Identifier::from([1u8; 32]),
            first_name: "Wisdom".to_string(),
            middle_name: "Madabuchukwu".to_string(),
            last_name: "Ogwu".to_string(),
            message: Some("Cantaloupe is the best fruit under the sun".to_string()),
            age: 20,
        };

        let person_1_updated = Person {
            id: Identifier::from([1u8; 32]),
            owner_id: Identifier::from([1u8; 32]),
            first_name: "Wisdom".to_string(),
            middle_name: "Madabuchukwu".to_string(),
            last_name: "Ogwu".to_string(),
            message: Some("My apples are safe".to_string()),
            age: 22,
        };

        apply_person(
            &drive,
            &contract,
            BlockInfo::default(),
            &person_0_original,
            true,
            transaction.as_ref(),
        );
        apply_person(
            &drive,
            &contract,
            BlockInfo::default(),
            &person_1_original,
            true,
            transaction.as_ref(),
        );
        apply_person(
            &drive,
            &contract,
            BlockInfo::default_with_time(100),
            &person_0_updated,
            true,
            transaction.as_ref(),
        );
        apply_person(
            &drive,
            &contract,
            BlockInfo::default_with_time(100),
            &person_1_updated,
            true,
            transaction.as_ref(),
        );
    }

    #[test]
    fn test_update_complex_person_with_history_no_transaction_and_has_raw() {
        test_update_complex_person(true, false, true)
    }

    #[test]
    fn test_update_complex_person_with_history_no_transaction_and_get_raw() {
        test_update_complex_person(true, false, false)
    }

    #[test]
    fn test_update_complex_person_with_history_with_transaction_and_has_raw() {
        test_update_complex_person(true, true, true)
    }

    #[test]
    fn test_update_complex_person_with_history_with_transaction_and_get_raw() {
        test_update_complex_person(true, true, false)
    }

    #[test]
    fn test_update_complex_person_no_history_no_transaction_and_has_raw() {
        test_update_complex_person(false, false, true)
    }

    #[test]
    fn test_update_complex_person_no_history_no_transaction_and_get_raw() {
        test_update_complex_person(false, false, false)
    }

    #[test]
    fn test_update_complex_person_no_history_with_transaction_and_has_raw() {
        test_update_complex_person(false, true, true)
    }

    #[test]
    fn test_update_complex_person_no_history_with_transaction_and_get_raw() {
        test_update_complex_person(false, true, false)
    }

    #[test]
    fn test_update_document_without_apply_should_calculate_storage_fees() {
        let tmp_dir = TempDir::new().unwrap();

        let drive: Drive =
            Drive::open(&tmp_dir, None).expect("expected to open Drive successfully");

        drive
            .create_initial_state_structure(None)
            .expect("expected to create root tree successfully");

        // Create a contract

        let block_info = BlockInfo::default();
        let owner_id = dpp::identifier::Identifier::new([2u8; 32]);

        let documents = platform_value!({
            "niceDocument": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string"
                    }
                },
                "required": [
                    "$createdAt"
                ],
                "additionalProperties": false
            }
        });

        let protocol_version_validator = ProtocolVersionValidator::new(
            LATEST_VERSION,
            LATEST_VERSION,
            COMPATIBILITY_MAP.clone(),
        );

        let data_contract_validator =
            DataContractValidator::new(Arc::new(protocol_version_validator));

        let factory = DataContractFactory::new_with_entropy_generator(
            1,
            Arc::new(data_contract_validator),
            Box::new(TestEntropyGenerator::new()),
        );

        let contract = factory
            .create(owner_id, documents, None, None)
            .expect("data in fixture should be correct")
            .data_contract;

        drive
            .apply_contract(
                &contract,
                block_info.clone(),
                true,
                StorageFlags::optional_default_as_cow(),
                None,
            )
            .expect("should apply contract");

        // Create a document factory

        let protocol_version_validator = Arc::new(ProtocolVersionValidator::new(
            LATEST_VERSION,
            LATEST_VERSION,
            COMPATIBILITY_MAP.clone(),
        ));

        let document_validator = DocumentValidator::new(protocol_version_validator);

        let document_factory = DocumentFactory::new(
            1,
            document_validator,
            DataContractFetcherAndValidator::new(Arc::new(MockStateRepositoryLike::new())),
        );

        // Create a document

        let document_type_name = "niceDocument".to_string();

        let document_type = contract
            .document_type_for_name(document_type_name.as_str())
            .expect("expected document type");

        let mut document = document_factory
            .create_extended_document_for_state_transition(
                contract.clone(),
                owner_id,
                document_type_name.clone(),
                json!({ "name": "Ivan" }).into(),
            )
            .expect("should create a document")
            .document;

        let storage_flags = Some(Cow::Owned(StorageFlags::SingleEpochOwned(
            0,
            owner_id.to_buffer(),
        )));

        let document_info = DocumentRefInfo((&document, storage_flags.clone()));

        let create_fees = drive
            .add_document_for_contract(
                DocumentAndContractInfo {
                    owned_document_info: OwnedDocumentInfo {
                        document_info,
                        owner_id: Some(owner_id.to_buffer()),
                    },
                    contract: &contract,
                    document_type,
                },
                false,
                block_info,
                true,
                None,
            )
            .expect("should create document");

        assert_ne!(create_fees.storage_fee, 0);

        // Update the document in a second

        document.set("name", Value::Text("Ivaaaaaaaaaan!".to_string()));

        let block_info = BlockInfo::default_with_time(10000);

        let update_fees = drive
            .update_document_for_contract(
                &document,
                &contract,
                document_type,
                Some(owner_id.to_buffer()),
                block_info,
                false,
                storage_flags,
                None,
            )
            .expect("should update document");

        assert_ne!(update_fees.storage_fee, 0);
    }
}
