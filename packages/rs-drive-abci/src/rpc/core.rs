use std::collections::HashMap;
use crate::error::Error;
use dashcore::{Block, BlockHash};
use dashcore_rpc::dashcore_rpc_json::{ExtendedQuorumDetails, GetBestChainLockResult, QuorumInfoResult, QuorumListResult, QuorumType};
use dashcore_rpc::{Auth, Client, RpcApi};
use mockall::{automock, predicate::*};
use serde_json::Value;
use tenderdash_abci::proto::types::CoreChainLock;
use dpp::dashcore::QuorumHash;

/// Information returned by QuorumListExtended
pub type QuorumListExtendedInfo = HashMap<QuorumHash, ExtendedQuorumDetails>;

/// Core height must be of type u32 (Platform heights are u64)
pub type CoreHeight = u32;
/// Core RPC interface
#[automock]
pub trait CoreRPCLike {
    /// Get block hash by height
    fn get_block_hash(&self, height: CoreHeight) -> Result<BlockHash, Error>;

    /// Get block hash by height
    fn get_best_chain_lock(&self) -> Result<CoreChainLock, Error>;

    /// Get block by hash
    fn get_block(&self, block_hash: &BlockHash) -> Result<Block, Error>;

    /// Get block by hash in JSON format
    fn get_block_json(&self, block_hash: &BlockHash) -> Result<Value, Error>;

    /// Get list of quorums at a given height.
    ///
    /// See https://dashcore.readme.io/v19.0.0/docs/core-api-ref-remote-procedure-calls-evo#quorum-listextended
    fn get_quorum_listextended(
        &self,
        height: Option<CoreHeight>,
    ) -> Result<QuorumListResult<QuorumListExtendedInfo>, Error>;

    /// Get quorum information.
    ///
    /// See https://dashcore.readme.io/v19.0.0/docs/core-api-ref-remote-procedure-calls-evo#quorum-info
    fn get_quorum_info(
        &self,
        quorum_type: QuorumType,
        hash: &QuorumHash,
        o: Option<bool>,
    ) -> Result<QuorumInfoResult, Error>;
}

/// Default implementation of Dash Core RPC using DashCoreRPC client
pub struct DefaultCoreRPC {
    inner: Client,
}

impl DefaultCoreRPC {
    /// Create new instance
    pub fn open(url: &str, username: String, password: String) -> Result<Self, Error> {
        Ok(DefaultCoreRPC {
            inner: Client::new(url, Auth::UserPass(username, password))?,
        })
    }
}

impl CoreRPCLike for DefaultCoreRPC {
    fn get_block_hash(&self, height: u32) -> Result<BlockHash, Error> {
        self.inner.get_block_hash(height).map_err(Error::CoreRpc)
    }

    fn get_best_chain_lock(&self) -> Result<CoreChainLock, Error> {
        let GetBestChainLockResult {
            blockhash,
            height,
            signature,
            known_block,
        } = self.inner.get_best_chain_lock().map_err(Error::CoreRpc)?;
        Ok(CoreChainLock {
            core_block_height: height,
            core_block_hash: blockhash.to_vec(),
            signature,
        })
    }

    fn get_block(&self, block_hash: &BlockHash) -> Result<Block, Error> {
        self.inner.get_block(block_hash).map_err(Error::CoreRpc)
    }

    fn get_block_json(&self, block_hash: &BlockHash) -> Result<Value, Error> {
        self.inner
            .get_block_json(block_hash)
            .map_err(Error::CoreRpc)
    }
    fn get_quorum_listextended(
        &self,
        height: Option<CoreHeight>,
    ) -> Result<QuorumListResult<QuorumListExtendedInfo>, Error> {
        self.inner.get_quorum_listextended(height.map(|i| i as i64)).map_err(Error::CoreRpc)
    }

    fn get_quorum_info(
        &self,
        quorum_type: QuorumType,
        quorum_hash: &QuorumHash,
        include_sk_share: Option<bool>,
    ) -> Result<QuorumInfoResult, Error> {
        self.inner
            .get_quorum_info(quorum_type, quorum_hash, include_sk_share)
    }
}
