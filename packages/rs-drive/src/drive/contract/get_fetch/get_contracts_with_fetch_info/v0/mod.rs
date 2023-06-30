use std::collections::BTreeMap;
use std::sync::Arc;
use grovedb::TransactionArg;
use dpp::version::drive_versions::DriveVersion;
use crate::drive::contract::ContractFetchInfo;
use crate::drive::Drive;
use crate::error::Error;

impl Drive {
    /// Retrieves the specified contracts along with their fetch info.
    ///
    /// # Arguments
    ///
    /// * `contract_ids` - A slice of contract IDs as 32-byte arrays. The contract IDs are used to
    ///   fetch the corresponding contracts and their fetch info.
    /// * `add_to_cache_if_pulled` - A boolean indicating whether to add the fetched contracts to the
    ///   cache if they were pulled from storage.
    /// * `transaction` - A `TransactionArg` object representing the transaction to be used
    ///   for fetching the contracts.
    ///
    /// # Returns
    ///
    /// * `Result<BTreeMap<[u8; 32], Option<Arc<Contract>>>, Error>` - If successful,
    ///   returns a `BTreeMap` where the keys are the contract IDs and the values are `Option`s
    ///   containing `Arc`s to `Contract`s. If an error occurs during the contract fetching,
    ///   returns an `Error`.
    ///
    /// # Errors
    ///
    /// This function returns an error if the contract fetching fails.
    pub(super) fn get_contracts_with_fetch_info_v0(
        &self,
        contract_ids: &[[u8; 32]],
        add_to_cache_if_pulled: bool,
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<BTreeMap<[u8; 32], Option<Arc<ContractFetchInfo>>>, Error> {
        contract_ids
            .iter()
            .map(|contract_id| {
                Ok((
                    *contract_id,
                    self.get_contract_with_fetch_info(
                        *contract_id,
                        add_to_cache_if_pulled,
                        transaction,
                        drive_version,
                    )?,
                ))
            })
            .collect()
    }
}