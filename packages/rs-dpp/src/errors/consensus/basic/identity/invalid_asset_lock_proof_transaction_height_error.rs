use crate::consensus::basic::BasicError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::consensus::ConsensusError;

use bincode::{Decode, Encode};

#[derive(Error, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[error("Core chain locked height {proof_core_chain_locked_height:?} must be higher than block {transaction_height:?} with Asset Lock transaction")]
pub struct InvalidAssetLockProofTransactionHeightError {
    /*

    DO NOT CHANGE ORDER OF FIELDS WITHOUT INTRODUCING OF NEW VERSION

    */
    proof_core_chain_locked_height: u32,
    transaction_height: Option<u32>,
}

impl InvalidAssetLockProofTransactionHeightError {
    pub fn new(proof_core_chain_locked_height: u32, transaction_height: Option<u32>) -> Self {
        Self {
            proof_core_chain_locked_height,
            transaction_height,
        }
    }

    pub fn proof_core_chain_locked_height(&self) -> u32 {
        self.proof_core_chain_locked_height
    }

    pub fn transaction_height(&self) -> Option<u32> {
        self.transaction_height
    }
}

impl From<InvalidAssetLockProofTransactionHeightError> for ConsensusError {
    fn from(err: InvalidAssetLockProofTransactionHeightError) -> Self {
        Self::BasicError(BasicError::InvalidAssetLockProofTransactionHeightError(err))
    }
}
