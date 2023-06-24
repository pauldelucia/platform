use dpp::identity::TimestampMillis;
use dpp::version::PlatformVersion;
use drive::grovedb::TransactionArg;
use crate::error::Error;
use crate::error::execution::ExecutionError;
use crate::platform_types::platform::Platform;
use crate::platform_types::system_identity_public_keys::SystemIdentityPublicKeys;

pub mod v0;

impl<C> Platform<C> {
    /// Creates trees and populates them with necessary identities, contracts and documents
    pub fn create_genesis_state(
        &self,
        genesis_time: TimestampMillis,
        system_identity_public_keys: SystemIdentityPublicKeys,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<(), Error> {
        match platform_version.drive_abci.methods.initialization.create_genesis_state {
            0 => self.create_genesis_state_v0(genesis_time, system_identity_public_keys, transaction, platform_version),
            version => Error::Execution(ExecutionError::UnknownVersionMismatch {
                method: "create_genesis_state".to_string(),
                known_versions: vec![0],
                received: version,
            })
        }
    }
}