use thiserror::Error;

use crate::consensus::signature::signature_error::SignatureError;
use crate::consensus::ConsensusError;
use crate::identity::KeyID;

use serde::{Deserialize, Serialize};

use bincode::{Decode, Encode};

#[derive(Error, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
#[error("Public key {public_key_id} doesn't exist")]
pub struct MissingPublicKeyError {
    /*

    DO NOT CHANGE ORDER OF FIELDS WITHOUT INTRODUCING OF NEW VERSION

    */
    public_key_id: KeyID,
}

impl MissingPublicKeyError {
    pub fn new(public_key_id: KeyID) -> Self {
        Self { public_key_id }
    }

    pub fn public_key_id(&self) -> KeyID {
        self.public_key_id
    }
}

impl From<MissingPublicKeyError> for ConsensusError {
    fn from(err: MissingPublicKeyError) -> Self {
        Self::SignatureError(SignatureError::MissingPublicKeyError(err))
    }
}
