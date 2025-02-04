use crate::drive::batch::transitions::document::DriveHighLevelDocumentOperationConverter;
use crate::drive::batch::DriveOperation::DocumentOperation;
use crate::drive::batch::{DocumentOperationType, DriveOperation};
use crate::drive::flags::StorageFlags;
use crate::drive::object_size_info::DocumentInfo::DocumentOwnedInfo;
use crate::drive::object_size_info::OwnedDocumentInfo;
use crate::error::Error;
use dpp::block::epoch::Epoch;

use dpp::document::document_transition::{
    DocumentBaseTransitionAction, DocumentReplaceTransitionAction,
};
use dpp::document::Document;
use dpp::prelude::Identifier;
use std::borrow::Cow;

impl DriveHighLevelDocumentOperationConverter for DocumentReplaceTransitionAction {
    fn into_high_level_document_drive_operations<'a>(
        self,
        epoch: &Epoch,
        owner_id: Identifier,
    ) -> Result<Vec<DriveOperation<'a>>, Error> {
        let DocumentReplaceTransitionAction {
            base,
            revision,
            created_at,
            updated_at,
            data,
        } = self;

        let DocumentBaseTransitionAction {
            id,
            document_type_name,
            data_contract_id,
            //todo: should we use the contract?
            ..
        } = base;

        let document = Document {
            id,
            owner_id,
            properties: data,
            revision: Some(revision),
            created_at,
            updated_at,
        };

        let storage_flags = StorageFlags::new_single_epoch(epoch.index, Some(owner_id.to_buffer()));

        let mut drive_operations = vec![];
        drive_operations.push(DocumentOperation(DocumentOperationType::UpdateDocument {
            owned_document_info: OwnedDocumentInfo {
                document_info: DocumentOwnedInfo((document, Some(Cow::Owned(storage_flags)))),
                owner_id: Some(owner_id.into_buffer()),
            },
            contract_id: data_contract_id,
            document_type_name: Cow::Owned(document_type_name),
        }));

        Ok(drive_operations)
    }
}
