use std::collections::BTreeMap;
use dashcore_rpc::dashcore::ProTxHash;
use dashcore_rpc::dashcore_rpc_json::{MasternodeListDiff, MasternodeListItem};
use dpp::block::block_info::BlockInfo;
use dpp::version::PlatformVersion;
use drive::grovedb::Transaction;
use crate::error::Error;
use crate::error::execution::ExecutionError;
use crate::platform_types::platform::Platform;
use crate::platform_types::platform_state::PlatformState;
use crate::rpc::core::CoreRPCLike;

mod v0;


impl<C> Platform<C>
    where
        C: CoreRPCLike,
{
    /// Update of the masternode identities
    pub(in crate::execution) fn update_masternode_identities(
        &self,
        masternode_diff: MasternodeListDiff,
        removed_masternodes: &BTreeMap<ProTxHash, MasternodeListItem>,
        block_info: &BlockInfo,
        platform_state: Option<&PlatformState>,
        transaction: &Transaction,
        platform_version: &PlatformVersion,
    ) -> Result<(), Error> {
        match platform_version.drive_abci.methods.core_based_updates.update_masternode_identities {
            0 => self.update_masternode_identities_v0(masternode_diff, removed_masternodes, block_info, platform_state, transaction, platform_version),
            version => Error::Execution(ExecutionError::UnknownVersionMismatch {
                method: "update_masternode_identities".to_string(),
                known_versions: vec![0],
                received: version,
            })
        }
    }
}