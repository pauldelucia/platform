use crate::consensus::basic::BasicError;
use crate::consensus::ConsensusError;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[error("Asset Lock Transaction Output with index ${output_index} not found")]
pub struct IdentityAssetLockTransactionOutputNotFoundError {
    /*

    DO NOT CHANGE ORDER OF FIELDS WITHOUT INTRODUCING OF NEW VERSION

    */
    output_index: usize,
}

impl IdentityAssetLockTransactionOutputNotFoundError {
    pub fn new(output_index: usize) -> Self {
        Self { output_index }
    }

    pub fn output_index(&self) -> usize {
        self.output_index
    }
}

impl From<IdentityAssetLockTransactionOutputNotFoundError> for ConsensusError {
    fn from(err: IdentityAssetLockTransactionOutputNotFoundError) -> Self {
        Self::BasicError(BasicError::IdentityAssetLockTransactionOutputNotFoundError(
            err,
        ))
    }
}
