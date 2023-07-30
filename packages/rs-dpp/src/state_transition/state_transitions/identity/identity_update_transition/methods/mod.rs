mod v0;
pub use v0::*;

use crate::identity::signer::Signer;
use crate::identity::{Identity, IdentityPublicKey, KeyID, PartialIdentity};
use crate::prelude::{AssetLockProof, Revision, TimestampMillis};
use crate::state_transition::identity_update_transition::v0::IdentityUpdateTransitionV0;
use crate::state_transition::identity_update_transition::IdentityUpdateTransition;
use crate::state_transition::public_key_in_creation::IdentityPublicKeyInCreation;
use crate::version::FeatureVersion;
use crate::{BlsModule, NonConsensusError, ProtocolError};
use platform_value::{Bytes32, Identifier};

impl IdentityUpdateTransitionMethodsV0 for IdentityUpdateTransition {
    #[cfg(feature = "state-transition-signing")]
    fn try_from_identity_with_signer<S: Signer>(
        identity: &Identity,
        master_public_key_id: &KeyID,
        add_public_keys: Vec<IdentityPublicKey>,
        disable_public_keys: Vec<KeyID>,
        public_keys_disabled_at: Option<u64>,
        signer: &S,
        version: FeatureVersion,
    ) -> Result<Self, ProtocolError> {
        match version {
            0 => Ok(IdentityUpdateTransitionV0::try_from_identity_with_signer(
                identity,
                master_public_key_id,
                add_public_keys,
                disable_public_keys,
                public_keys_disabled_at,
                signer,
                version,
            )?
            .into()),
            v => Err(ProtocolError::UnknownVersionError(format!(
                "Unknown IdentityUpdateTransition version for try_from_identity_with_signer {v}"
            ))),
        }
    }
}
