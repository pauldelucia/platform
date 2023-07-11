use std::sync::Arc;
use grovedb::TransactionArg;
use dpp::block::epoch::Epoch;
use crate::fee::calculate_fee;
use dpp::fee::fee_result::FeeResult;
use dpp::version::drive_versions::DriveVersion;
use crate::drive::contract::ContractFetchInfo;
use crate::drive::Drive;
use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;
use crate::fee::op::LowLevelDriveOperation::{CalculatedCostOperation, PreCalculatedFeeResult};

impl Drive {
    /// Retrieves the specified contract.
    ///
    /// # Arguments
    ///
    /// * `contract_id` - A contract ID as a 32-byte array. The contract ID is used to
    ///   fetch the corresponding contract.
    /// * `add_to_cache_if_pulled` - A boolean indicating whether to add the fetched contract to the
    ///   cache if it was pulled from storage.
    /// * `transaction` - A `TransactionArg` object representing the transaction to be used
    ///   for fetching the contract.
    ///
    /// # Returns
    /// TODO(doc): returns ContractFetchInfo, not Contract. Do we really need to write it in the doc since it's
    /// provided in the return argument?
    /// * `Result<Option<Arc<Contract>>, Error>` - If successful, returns an `Option` containing a
    ///   reference to the fetched `Contract`. If an error occurs during the contract fetching,
    ///   returns an `Error`.
    ///
    /// # Errors
    ///
    /// This function returns an error if the contract fetching fails.
    pub(super) fn get_contract_with_fetch_info_v0(
        &self,
        contract_id: [u8; 32],
        add_to_cache_if_pulled: bool,
        transaction: TransactionArg,
    ) -> Result<Option<Arc<ContractFetchInfo>>, Error> {
        self.get_contract_with_fetch_info_and_add_to_operations_v0(
            contract_id,
            None,
            add_to_cache_if_pulled,
            transaction,
            &mut vec![],
        )
    }

    /// Retrieves the specified contract along with its fetch info and calculates the fee if an epoch is provided.
    /// TODO(doc): probably it worth explaining reasoning of why we having fee for fetch method.
    /// # Arguments
    ///
    /// * `contract_id` - A contract ID as a 32-byte array. The contract ID is used to
    ///   fetch the corresponding contract and its fetch info.
    /// * `epoch` - An optional reference to an `Epoch` object. If provided, the function calculates
    ///   the fee for the contract operations.
    /// * `add_to_cache_if_pulled` - A boolean indicating whether to add the fetched contract to the
    ///   cache if it was pulled from storage.
    /// * `transaction` - A `TransactionArg` object representing the transaction to be used
    ///
    ///   for fetching the contract.
    ///
    /// # Returns
    ///  // TODO(doc): Do we really need to write return type in the Doc?
    /// * `Result<(Option<FeeResult>, Option<Arc<ContractFetchInfo>>), Error>` - If successful,
    ///   returns a tuple containing an `Option` with the `FeeResult` (if an epoch was provided) and
    ///   an `Option` containing an `Arc` to the fetched `ContractFetchInfo`. If an error occurs
    ///   during the contract fetching or fee calculation, returns an `Error`.
    ///
    /// # Errors
    ///
    /// This function returns an error if the contract fetching or fee calculation fails.
    pub(super) fn get_contract_with_fetch_info_and_fee_v0(
        &self,
        contract_id: [u8; 32],
        epoch: Option<&Epoch>,
        add_to_cache_if_pulled: bool,
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<(Option<FeeResult>, Option<Arc<ContractFetchInfo>>), Error> {
        let mut drive_operations: Vec<LowLevelDriveOperation> = Vec::new();

        let contract_fetch_info = self.get_contract_with_fetch_info_and_add_to_operations_v0(
            contract_id,
            epoch,
            add_to_cache_if_pulled,
            transaction,
            &mut drive_operations,
            drive_version,
        )?;
        let fee_result = epoch.map_or(Ok(None), |epoch| {
            calculate_fee(None, Some(drive_operations), epoch).map(Some)
        })?;
        Ok((fee_result, contract_fetch_info))
    }

    /// Returns the contract with fetch info and operations with the given ID.
    pub(super) fn get_contract_with_fetch_info_and_add_to_operations_v0(
        &self,
        contract_id: [u8; 32],
        epoch: Option<&Epoch>,
        add_to_cache_if_pulled: bool,
        transaction: TransactionArg,
        drive_operations: &mut Vec<LowLevelDriveOperation>,
        drive_version: &DriveVersion,
    ) -> Result<Option<Arc<ContractFetchInfo>>, Error> {
        let cache = self.cache.read().unwrap();

        match cache
            .cached_contracts
            .get(contract_id, transaction.is_some())
        {
            None => {
                let maybe_contract_fetch_info = self.fetch_contract_and_add_operations(
                    contract_id,
                    epoch,
                    transaction,
                    drive_operations,
                    drive_version,
                )?;

                if add_to_cache_if_pulled {
                    // Store a contract in cache if present
                    if let Some(contract_fetch_info) = &maybe_contract_fetch_info {
                        drop(cache);
                        let mut cache = self.cache.write().unwrap();
                        cache
                            .cached_contracts
                            .insert(Arc::clone(contract_fetch_info), transaction.is_some());
                    };
                }
                Ok(maybe_contract_fetch_info)
            }
            Some(contract_fetch_info) => {
                // we only need to pay if epoch is set
                if let Some(epoch) = epoch {
                    let fee = if let Some(known_fee) = &contract_fetch_info.fee {
                        known_fee.clone()
                    } else {
                        // we need to calculate new fee
                        let op = vec![CalculatedCostOperation(contract_fetch_info.cost.clone())];
                        let fee = calculate_fee(None, Some(op), epoch)?;

                        let updated_contract_fetch_info = Arc::new(ContractFetchInfo {
                            contract: contract_fetch_info.contract.clone(),
                            storage_flags: contract_fetch_info.storage_flags.clone(),
                            cost: contract_fetch_info.cost.clone(),
                            fee: Some(fee.clone()),
                        });
                        drop(cache);
                        let mut cache = self.cache.write().unwrap();
                        // we override the cache for the contract as the fee is now calculated
                        cache
                            .cached_contracts
                            .insert(updated_contract_fetch_info, transaction.is_some());

                        fee
                    };
                    drive_operations.push(PreCalculatedFeeResult(fee));
                }
                Ok(Some(contract_fetch_info))
            }
        }
    }
}