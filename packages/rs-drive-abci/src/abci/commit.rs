//! Processing of commits generated by Tenderdash
use crate::execution::finalize_block_cleaned_request::{CleanedBlockId, CleanedCommitInfo};

use dashcore_rpc::dashcore_rpc_json::QuorumType;
use dpp::bls_signatures;
use dpp::validation::{SimpleValidationResult, ValidationResult};
use tenderdash_abci::proto;
use tenderdash_abci::proto::abci::CommitInfo;
use tenderdash_abci::proto::types::BlockId;
use tenderdash_abci::signatures::SignDigest;

use super::AbciError;

/// Represents block commit
#[derive(Clone, Debug)]
pub struct Commit {
    /// Commit in Tenderdash format
    pub inner: proto::types::Commit,
    /// ID of chain used to sign this commit
    pub chain_id: String,
    /// Type of quorum used to sign this commit
    pub quorum_type: QuorumType,
}

impl Commit {
    /// Create new Commit struct based on commit info and block id received from Tenderdash
    pub fn new_from_cleaned(
        ci: CleanedCommitInfo,
        block_id: CleanedBlockId,
        height: u64,
        quorum_type: QuorumType,
        chain_id: &str,
    ) -> Self {
        Self {
            chain_id: String::from(chain_id),
            quorum_type,

            inner: proto::types::Commit {
                block_id: Some(block_id.try_into().expect("cannot convert block id")),
                height: height as i64,
                round: ci.round as i32,
                // we need to "un-reverse" quorum hash, as it was reversed in [CleanedCommitInfo::try_from]
                quorum_hash: ci.quorum_hash.to_vec(),
                threshold_block_signature: ci.block_signature.to_vec(),
                threshold_vote_extensions: ci.threshold_vote_extensions.to_vec(),
            },
        }
    }

    /// Create new Commit struct based on commit info and block id received from Tenderdash
    pub fn new(
        ci: CommitInfo,
        block_id: BlockId,
        height: u64,
        quorum_type: QuorumType,
        chain_id: &str,
    ) -> Self {
        Self {
            chain_id: String::from(chain_id),
            quorum_type,

            inner: proto::types::Commit {
                block_id: Some(block_id),
                height: height as i64,
                round: ci.round,
                quorum_hash: ci.quorum_hash,
                threshold_block_signature: ci.block_signature,
                threshold_vote_extensions: ci.threshold_vote_extensions,
            },
        }
    }

    /// Verify all signatures using provided public key.
    ///
    /// ## Return value
    ///
    /// * Ok(true) when all signatures are correct
    /// * Ok(false) when at least one signature is invalid
    /// * Err(e) on error
    pub fn verify_signature(
        &self,
        signature: &[u8; 96],
        public_key: &bls_signatures::PublicKey,
    ) -> SimpleValidationResult<AbciError> {
        if signature == &[0; 96] {
            return ValidationResult::new_with_error(AbciError::BadRequest(
                "commit signature not initialized".to_string(),
            ));
        }
        // We could have received a fake commit, so signature validation needs to be returned if error as a simple validation result
        let signature = match bls_signatures::Signature::from_bytes(signature).map_err(|e| {
            AbciError::BlsErrorOfTenderdashThresholdMechanism(
                e,
                "verification of a commit signature".to_string(),
            )
        }) {
            Ok(signature) => signature,
            Err(e) => return ValidationResult::new_with_error(e),
        };

        //todo: maybe cache this to lower the chance of a hashing based attack (forcing the
        // same calculation each time)
        let quorum_hash = &self.inner.quorum_hash[..]
            .try_into()
            .expect("invalid quorum hash length");

        let hash = match self
            .inner
            .sign_digest(
                &self.chain_id,
                self.quorum_type as u8,
                quorum_hash,
                self.inner.height,
                self.inner.round,
            )
            .map_err(AbciError::Tenderdash)
        {
            Ok(hash) => hash,
            Err(e) => return ValidationResult::new_with_error(e),
        };

        match public_key.verify(&signature, &hash) {
            true => ValidationResult::default(),
            false => ValidationResult::new_with_error(AbciError::BadCommitSignature(format!(
                "commit signature {} is wrong",
                hex::encode(signature.to_bytes().as_slice())
            ))),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::execution::finalize_block_cleaned_request::CleanedCommitInfo;

    use super::Commit;
    use dashcore_rpc::{
        dashcore::hashes::sha256, dashcore::hashes::Hash, dashcore_rpc_json::QuorumType,
    };
    use dpp::bls_signatures::PublicKey;
    use tenderdash_abci::proto::types::{BlockId, PartSetHeader, StateId};
    use tenderdash_abci::signatures::{SignBytes, SignDigest};

    /// Given a commit info and a signature, check that the signature is verified correctly
    #[test]
    fn test_commit_verify() {
        const HEIGHT: i64 = 12345;
        const ROUND: u32 = 2;
        const CHAIN_ID: &str = "test_chain_id";

        const QUORUM_HASH: [u8; 32] = [0u8; 32];

        let ci = CleanedCommitInfo {
            round: ROUND,
            quorum_hash: QUORUM_HASH,
            block_signature: [0u8; 96],
            threshold_vote_extensions: Vec::new(),
        };
        let app_hash = [1u8, 2, 3, 4].repeat(8);

        let state_id = StateId {
            height: HEIGHT as u64,
            app_hash,
            app_version: 1,
            core_chain_locked_height: 3,
            time: Some(tenderdash_abci::proto::google::protobuf::Timestamp {
                seconds: 0,
                nanos: 0,
            }),
        };

        let block_id = BlockId {
            hash: sha256::Hash::hash("blockID_hash".as_bytes())
                .to_byte_array()
                .to_vec(),
            part_set_header: Some(PartSetHeader {
                total: 1000000,
                hash: sha256::Hash::hash("blockID_part_set_header_hash".as_bytes())
                    .to_byte_array()
                    .to_vec(),
            }),
            state_id: state_id.sha256(CHAIN_ID, HEIGHT, ROUND as i32).unwrap(),
        };
        let pubkey = hex::decode(
            "b7b76cbef11f48952b4c9778b0cd1e27948c6438c0480e69ce78\
            dc4748611f4463389450a6898f91b08f1de666934324",
        )
        .unwrap();

        let pubkey = PublicKey::from_bytes(pubkey.as_slice()).unwrap();
        let signature = hex::decode("95e4a532ccb549cd4feca372b61dd2a5dedea2bb5c33ac22d70e310f\
            7e38126b21029c29e6af6d00462b7c6f5e47047414dbfb2e1008fa0969a246bc38b61e96edddea9c35a01670b0ae45f0\
            8a2626b251bb2a8e937547e65994f2c72d2e8f4e").unwrap();

        let commit = Commit::new_from_cleaned(
            ci,
            block_id.try_into().unwrap(),
            HEIGHT as u64,
            QuorumType::LlmqTest,
            CHAIN_ID,
        );

        let expect_sign_bytes = hex::decode("0200000039300000000000000200000000000000\
            35117edfe49351da1e81d1b0f2edfa0b984a7508958870337126efb352f1210711ae5fef92053e8998c37cb4\
            915968cadfbd2af4fa176b77ade0dadc74028fc5746573745f636861696e5f6964").unwrap();
        let expect_sign_id =
            hex::decode("6f3cb0168cfaf3d9806be8a9eaa85d6ac10e2d32ce02e6a965a66f6c598b06cf")
                .unwrap();
        assert_eq!(
            expect_sign_bytes,
            commit
                .inner
                .sign_bytes(CHAIN_ID, HEIGHT, ROUND as i32)
                .unwrap()
        );
        assert_eq!(
            expect_sign_id,
            commit
                .inner
                .sign_digest(
                    CHAIN_ID,
                    QuorumType::LlmqTest as u8,
                    &QUORUM_HASH,
                    HEIGHT,
                    ROUND as i32
                )
                .unwrap()
        );
        assert!(commit
            .verify_signature(
                &signature.clone().try_into().expect("expected 96 bytes"),
                &pubkey
            )
            .is_valid());

        // mutate data and ensure it is invalid
        let mut commit = commit;
        commit.chain_id = "invalid".to_string();
        assert!(!commit
            .verify_signature(&signature.try_into().expect("expected 96 bytes"), &pubkey)
            .is_valid());
    }
}
