use std::borrow::Cow;
use std::collections::HashMap;
use grovedb::batch::KeyInfoPath;
use grovedb::{EstimatedLayerInformation, TransactionArg};
use dpp::block::block_info::BlockInfo;
use dpp::data_contract::DataContract;
use dpp::state_transition::fee::fee_result::FeeResult;
use crate::drive::Drive;
use crate::drive::flags::StorageFlags;
use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;
use dpp::serialization_traits::{PlatformDeserializable, PlatformSerializable};
use dpp::version::drive_versions::DriveVersion;

impl Drive {
    /// Applies a contract and returns the fee for applying.
    ///
    /// This function applies a given contract to the storage. If the contract already exists,
    /// an update is performed; otherwise, a new contract is inserted. The fee for applying
    /// the contract is also calculated and returned.
    ///
    /// # Arguments
    ///
    /// * `contract` - A reference to the `Contract` to be applied.
    /// * `block_info` - A `BlockInfo` object containing information about the block where
    ///   the contract is being applied.
    /// * `apply` - A boolean indicating whether the contract should be applied (`true`) or not (`false`).
    /// * `storage_flags` - An optional `Cow<StorageFlags>` containing the storage flags for the contract.
    /// * `transaction` - A `TransactionArg` object representing the transaction to be used
    ///   for applying the contract.
    ///
    /// # Returns
    ///
    /// * `Result<FeeResult, Error>` - If successful, returns a `FeeResult` representing the fee
    ///   for applying the contract. If an error occurs during the contract application or fee calculation,
    ///   returns an `Error`.
    ///
    /// # Errors
    ///
    /// This function returns an error if the contract application or fee calculation fails.
    pub(super) fn apply_contract_v0(
        &self,
        contract: &DataContract,
        block_info: BlockInfo,
        apply: bool,
        storage_flags: Option<Cow<StorageFlags>>,
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<FeeResult, Error> {
        self.apply_contract_with_serialization(
            contract,
            contract.serialize()?,
            block_info,
            apply,
            storage_flags,
            transaction,
            drive_version
        )
    }

    /// Gets the operations for applying a contract
    /// If the contract already exists, we get operations for an update
    /// Otherwise we get operations for an insert
    pub(super) fn apply_contract_operations_v0(
        &self,
        contract: &DataContract,
        block_info: &BlockInfo,
        estimated_costs_only_with_layer_info: &mut Option<
            HashMap<KeyInfoPath, EstimatedLayerInformation>,
        >,
        storage_flags: Option<Cow<StorageFlags>>,
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<Vec<LowLevelDriveOperation>, Error> {
        let serialized_contract = contract.serialize().map_err(Error::Protocol)?;
        self.apply_contract_with_serialization_operations(
            contract,
            serialized_contract,
            block_info,
            estimated_costs_only_with_layer_info,
            storage_flags,
            transaction,
            drive_version,
        )
    }
}