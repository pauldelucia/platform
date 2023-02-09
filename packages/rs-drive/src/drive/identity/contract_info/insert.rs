use crate::drive::flags::StorageFlags;
use crate::drive::grove_operations::BatchInsertTreeApplyType::StatefulBatchInsertTree;
use crate::drive::identity::IdentityRootStructure::IdentityContractInfo;
use crate::drive::identity::{
    identity_contract_info_root_path_vec, identity_key_location_within_identity_vec,
    identity_path_vec,
};
use crate::drive::object_size_info::PathKeyInfo;
use crate::drive::Drive;
use crate::error::identity::IdentityError;
use crate::error::Error;
use crate::fee::op::DriveOperation;
use crate::fee_pools::epochs::Epoch;
use dpp::identity::contract_bounds::ContractBounds;
use dpp::identity::{IdentityPublicKey, KeyID};
use dpp::prelude::DataContract;
use grovedb::batch::KeyInfoPath;
use grovedb::{EstimatedLayerInformation, TransactionArg};
use integer_encoding::VarInt;
use std::collections::{BTreeMap, HashMap};
use dpp::data_contract::document_type::DocumentType;
use dpp::data_contract::DriveContractExt;

pub enum ContractApplyInfo<'a> {
    /// The root_id is either a contract id or an owner id
    /// It is a contract id for in the case of contract bound keys or contract
    /// document bound keys
    /// In the case
    ContractBased {
        root_id: &'a DataContract
        document_type_keys: BTreeMap<&'a str, Vec<KeyID>>,
        contract_keys: Vec<KeyID>,
    },
}

impl ContractApplyInfo {
    fn new_from_single_key(
        key_id: keyID,
        contract_bounds: &ContractBounds,
        drive: &Drive,
        epoch: &Epoch,
        transaction: TransactionArg,
        drive_operations: &mut Vec<DriveOperation>,
    ) -> Result<Self, Error> {
        let contract_id = contract_bounds.identifier().to_buffer();
        // we are getting with fetch info to add the cost to the drive operations
        let maybe_contract_fetch_info = drive.get_contract_with_fetch_info_and_add_to_operations(
            contract_id,
            Some(epoch),
            transaction,
            drive_operations,
        )?;
        let Some(contract_fetch_info) = maybe_contract_fetch_info else {
            return Err(Error::Identity(IdentityError::IdentityKeyBoundsError("Contract for key bounds not found")));
        };
        let contract = &contract_fetch_info.contract;
        match contract_bounds {
            ContractBounds::SingleContract { .. } => Ok(ContractApplyInfo {
                root_id: contract,
                document_type_keys: Default::default(),
                contract_keys: vec![key_id],
            }),
            ContractBounds::SingleContractDocumentType { document_type, .. } => {
                let document_type = contract
                    .document_type_for_name(document_type)
                    .map_err(Error::Protocol)?;
                Ok(ContractApplyInfo {
                    root_id: contract,
                    document_type_keys: BTreeMap::from([(&document_type.name, vec![key_id])]),
                    contract_keys: vec![],
                })
            }
            ContractBounds::MultipleContractsOfSameOwner { .. } => Ok(ContractApplyInfo {
                root_id: contract,
                document_type_keys: Default::default(),
                contract_keys: vec![key_id],
            }),
        }
    }
}

impl Drive {
    pub(crate) fn add_potential_contract_info_for_contract_bounded_key(
        &self,
        identity_id: [u8; 32],
        identity_key: &IdentityPublicKey,
        epoch: &Epoch,
        estimated_costs_only_with_layer_info: &mut Option<
            HashMap<KeyInfoPath, EstimatedLayerInformation>,
        >,
        transaction: TransactionArg,
        drive_operations: &mut Vec<DriveOperation>,
    ) -> Result<(), Error> {
        if let Some(contract_bounds) = &identity_key.contract_bounds {
            // We need to get the contract
            let contract_apply_info = ContractApplyInfo::new_from_single_key(
                identity_key.id,
                contract_bounds,
                self,
                epoch,
                transaction,
                drive_operations,
            )?;
            self.add_contract_info_operations(
                identity_id,
                vec![contract_apply_info],
                estimated_costs_only_with_layer_info,
                transaction,
                drive_operations,
            )
        }
        Ok(())
    }

    /// Adds the contract info operations
    pub(crate) fn add_contract_info_operations(
        &self,
        identity_id: [u8; 32],
        contract_infos: Vec<ContractApplyInfo>,
        estimated_costs_only_with_layer_info: &mut Option<
            HashMap<KeyInfoPath, EstimatedLayerInformation>,
        >,
        transaction: TransactionArg,
        drive_operations: &mut Vec<DriveOperation>,
    ) -> Result<(), Error> {
        let identity_path = identity_path_vec(identity_id.as_slice());
        // we insert the contract root tree if it doesn't exist already
        self.batch_insert_empty_tree_if_not_exists_check_existing_operations(
            PathKeyInfo::<0>::PathKey((identity_path, vec![IdentityContractInfo as u8])),
            None,
            StatefulBatchInsertTree,
            transaction,
            drive_operations,
        )?;

        for contract_infos in contract_infos.into_iter() {
            let ContractApplyInfo {
                root_id: contract,
                document_type_keys,
                contract_keys,
            } = contract_infos;
            self.batch_insert_empty_tree_if_not_exists_check_existing_operations(
                PathKeyInfo::<0>::PathKey((
                    identity_contract_info_root_path_vec(identity_id.as_slice()),
                    contract.id.to_buffer_vec(),
                )),
                Some(&storage_flags),
                StatefulBatchInsertTree,
                transaction,
                drive_operations,
            )?;
            for key_id in contract_keys {
                // we need to add a reference to the key
                let key_id_bytes = key_id.encode_var_vec();
                let key_reference =
                    identity_key_location_within_identity_vec(key_id_bytes.as_slice());
                self.batch_insert_empty_tree_if_not_exists_check_existing_operations(
                    PathKeyInfo::<0>::PathKey((
                        identity_contract_info_root_path_vec(identity_id.as_slice()),
                        contract.id.to_buffer_vec(),
                    )),
                    Some(&storage_flags),
                    StatefulBatchInsertTree,
                    transaction,
                    drive_operations,
                )?;
            }
        }
        Ok(())
    }
}
