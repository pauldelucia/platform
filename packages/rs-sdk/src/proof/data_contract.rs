use std::collections::BTreeMap;

use dpp::prelude::DataContract;
use drive::drive::verify::RootHash;

use crate::error::Error;

pub struct DataContractProof {}

impl DataContractProof {
    /// Verifies that the contract is included in the proof.
    ///
    /// # Parameters
    ///
    /// - `proof`: A byte slice representing the proof to be verified.
    /// - `contract_known_keeps_history`: An optional boolean indicating whether the contract keeps a history.
    /// - `is_proof_subset`: A boolean indicating whether to verify a subset of a larger proof.
    /// - `contract_id`: The contract's unique identifier.
    ///
    /// # Returns
    ///
    /// Returns a `Result` with a tuple of `RootHash` and `Option<DataContract>`. The `Option<DataContract>`
    /// represents the verified contract if it exists.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if:
    ///
    /// - The proof is corrupted.
    /// - The GroveDb query fails.
    pub fn contract(
        &self,
        contract_id: [u8; 32],
        is_proof_subset: bool,
        contract_known_keeps_history: Option<bool>,
    ) -> Result<(RootHash, Option<DataContract>), Error> {
        todo!()
    }

    /// Verifies that the contract's history is included in the proof.
    ///
    /// # Parameters
    ///
    /// - `proof`: A byte slice representing the proof to be verified.
    /// - `contract_id`: The contract's unique identifier.
    /// - `start_at_date`: The start date for the contract's history.
    /// - `limit`: An optional limit for the number of items to be retrieved.
    /// - `offset`: An optional offset for the items to be retrieved.
    ///
    /// # Returns
    ///
    /// Returns a `Result` with a tuple of `RootHash` and `Option<BTreeMap<u64, DataContract>>`. The `Option<BTreeMap<u64, DataContract>>`
    /// represents a mapping from dates to contracts if it exists.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if:
    ///
    /// - The proof is corrupted.
    /// - The GroveDb query fails.
    /// - The contract serialization fails.
    pub fn contract_history(
        &self,
        contract_id: [u8; 32],
        start_at_date: u64,
        limit: Option<u16>,
        offset: Option<u16>,
    ) -> Result<(RootHash, Option<BTreeMap<u64, DataContract>>), Error> {
        todo!()
    }
}
