use crate::drive::flags::StorageFlags;
use crate::drive::grove_operations::BatchInsertTreeApplyType::StatefulBatchInsertTree;
use crate::drive::identity::IdentityRootStructure::IdentityContractInfo;
use crate::drive::identity::{identity_contract_info_root_path_vec, identity_path_vec};
use crate::drive::object_size_info::PathKeyInfo;
use crate::drive::Drive;
use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;
use dpp::block::epoch::Epoch;
use dpp::identity::IdentityPublicKey;
use dpp::version::PlatformVersion;
use grovedb::batch::KeyInfoPath;
use grovedb::{EstimatedLayerInformation, TransactionArg};
use std::collections::HashMap;

/// data contract apply info
#[allow(dead_code)]
pub enum DataContractApplyInfo {
    Keys(Vec<IdentityPublicKey>),
}

impl Drive {
    /// Adds contract information to an identity within the Dash Platform's GroveDB.
    ///
    /// This function generates a series of low-level drive operations needed to
    /// add contract information to an identity in GroveDB. These operations can be part of a batch operation.
    ///
    /// # Parameters
    ///
    /// - `identity_id`: A 32-byte array that uniquely identifies the identity.
    /// - `contract_infos`: A vector of tuples each containing a 32-byte contract ID and associated `DataContractApplyInfo`.
    /// - `epoch`: The current epoch in the Dash Platform.
    /// - `estimated_costs_only_with_layer_info`: Mutable reference to an optional hashmap that contains
    ///   layer information specifically for cost estimation.
    /// - `transaction`: The transaction arguments for the operation.
    /// - `platform_version`: Version information for the Dash Platform where the operation occurs.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<LowLevelDriveOperation>)`: A vector of low-level drive operations that should be applied to GroveDB.
    /// - `Err(Error)`: An error occurred, as defined by the custom `Error` type.
    ///
    /// # Errors
    ///
    /// - Errors may be propagated from `batch_insert_empty_tree_if_not_exists` and `add_new_keys_to_identity_operations`.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Assuming all types and Error variants are defined
    /// let identity_id: [u8; 32] = /* ... */;
    /// let contract_infos: Vec<([u8; 32], DataContractApplyInfo)> = /* ... */;
    /// let epoch: Epoch = /* ... */;
    /// let mut estimated_costs: Option<HashMap<KeyInfoPath, EstimatedLayerInformation>> = Some(HashMap::new());
    /// let transaction: TransactionArg = /* ... */;
    /// let platform_version: PlatformVersion = /* ... */;
    ///
    /// let result = drive_instance.add_contract_info_operations(
    ///     identity_id,
    ///     contract_infos,
    ///     &epoch,
    ///     &mut estimated_costs,
    ///     transaction,
    ///     &platform_version
    /// );
    /// ```
    #[allow(dead_code)]
    pub(crate) fn add_contract_info_operations(
        &self,
        identity_id: [u8; 32],
        contract_infos: Vec<([u8; 32], DataContractApplyInfo)>,
        epoch: &Epoch,
        estimated_costs_only_with_layer_info: &mut Option<
            HashMap<KeyInfoPath, EstimatedLayerInformation>,
        >,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<Vec<LowLevelDriveOperation>, Error> {
        let mut batch_operations: Vec<LowLevelDriveOperation> = vec![];
        let storage_flags = StorageFlags::SingleEpoch(epoch.index);
        let identity_path = identity_path_vec(identity_id.as_slice());
        // we insert the contract root tree if it doesn't exist already
        self.batch_insert_empty_tree_if_not_exists(
            PathKeyInfo::<0>::PathKey((identity_path, vec![IdentityContractInfo as u8])),
            Some(&storage_flags),
            StatefulBatchInsertTree,
            transaction,
            &mut None,
            &mut batch_operations,
            &platform_version.drive,
        )?;

        let identity_contract_root_path =
            identity_contract_info_root_path_vec(identity_id.as_slice());

        for (contract_id, contract_info) in contract_infos.into_iter() {
            self.batch_insert_empty_tree_if_not_exists(
                PathKeyInfo::<0>::PathKey((
                    identity_contract_root_path.clone(),
                    contract_id.to_vec(),
                )),
                Some(&storage_flags),
                StatefulBatchInsertTree,
                transaction,
                &mut None,
                &mut batch_operations,
                &platform_version.drive,
            )?;
            match contract_info {
                DataContractApplyInfo::Keys(keys) => {
                    self.add_new_keys_to_identity_operations(
                        identity_id,
                        keys,
                        vec![],
                        false,
                        estimated_costs_only_with_layer_info,
                        transaction,
                        platform_version,
                    )?;
                }
            }
        }
        Ok(batch_operations)
    }
}
