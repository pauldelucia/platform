// MIT LICENSE
//
// Copyright (c) 2021 Dash Core Group
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
//

//! Execution Tests
//!

extern crate core;

use dashcore_rpc::dashcore::QuorumHash;

use dpp::bls_signatures::PrivateKey as BlsPrivateKey;

use drive_abci::test::helpers::setup::TestPlatformBuilder;
use drive_abci::{config::PlatformConfig, test::helpers::setup::TempPlatform};
use frequency::Frequency;

use std::collections::BTreeMap;

use strategy::{ChainExecutionOutcome, ChainExecutionParameters, Strategy, StrategyRandomness};

mod execution;
mod failures;
mod frequency;
mod masternode_list_item_helpers;
mod masternodes;
mod operations;
mod query;
mod signer;
mod strategy;
mod transitions;
mod upgrade_fork_tests;
mod verify_state_transitions;

pub type BlockHeight = u64;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::{continue_chain_for_strategy, run_chain_for_strategy};
    use crate::operations::DocumentAction::DocumentActionReplace;
    use crate::operations::{
        DocumentAction, DocumentOp, IdentityUpdateOp, Operation, OperationType,
    };
    use crate::query::QueryStrategy;
    use crate::strategy::MasternodeListChangesStrategy;
    use dashcore_rpc::dashcore::hashes::Hash;
    use dashcore_rpc::dashcore::BlockHash;
    use dashcore_rpc::dashcore_rpc_json::ExtendedQuorumDetails;
    use dpp::data_contract::extra::common::json_document_to_created_contract;
    use dpp::util::hash::hash_to_hex_string;
    use drive_abci::config::PlatformTestConfig;
    use drive_abci::rpc::core::QuorumListExtendedInfo;
    use itertools::Itertools;
    use tenderdash_abci::proto::abci::{RequestInfo, ResponseInfo};
    use tenderdash_abci::proto::types::CoreChainLock;
    use tenderdash_abci::Application;

    pub fn generate_quorums_extended_info(n: u32) -> QuorumListExtendedInfo {
        let mut quorums = QuorumListExtendedInfo::new();

        for i in 0..n {
            let i_bytes = [i as u8; 32];

            let hash = QuorumHash::from_inner(i_bytes);

            let details = ExtendedQuorumDetails {
                creation_height: i,
                health_ratio: (i as f32) / (n as f32),
                mined_block_hash: BlockHash::from_slice(&i_bytes).unwrap(),
                num_valid_members: i,
                quorum_index: Some(i),
            };

            if let Some(v) = quorums.insert(hash, details) {
                panic!("duplicate record {:?}={:?}", hash, v)
            }
        }
        quorums
    }

    #[test]
    fn run_chain_nothing_happening() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![],
            identities_inserts: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: false,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 3000,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();

        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        run_chain_for_strategy(&mut platform, 100, strategy, config, 15);
    }

    #[test]
    fn run_chain_block_signing() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![],
            identities_inserts: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: false,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 3000,
            testing_configs: PlatformTestConfig::default(),
            ..Default::default()
        };
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();

        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        run_chain_for_strategy(&mut platform, 50, strategy, config, 13);
    }

    #[test]
    fn run_chain_stop_and_restart() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![],
            identities_inserts: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: false,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 3000,
            testing_configs: PlatformTestConfig::default(),
            ..Default::default()
        };
        let TempPlatform {
            mut platform,
            tempdir: _,
        } = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();

        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });

        let ChainExecutionOutcome {
            abci_app,
            proposers,
            quorums,
            current_quorum_hash,
            current_proposer_versions,
            end_time_ms,
            ..
        } = run_chain_for_strategy(&mut platform, 15, strategy.clone(), config.clone(), 40);

        let known_root_hash = abci_app
            .platform
            .drive
            .grove
            .root_hash(None)
            .unwrap()
            .expect("expected root hash");

        abci_app
            .platform
            .recreate_state()
            .expect("expected to recreate state");

        let ResponseInfo {
            data: _,
            version: _,
            app_version: _,
            last_block_height,
            last_block_app_hash,
        } = abci_app
            .info(RequestInfo {
                version: "0.12.0".to_string(),
                block_version: 0,
                p2p_version: 0,
                abci_version: "0.20.0".to_string(),
            })
            .expect("expected to call info");

        assert_eq!(last_block_height, 15);
        assert_eq!(last_block_app_hash, known_root_hash);

        let block_start = abci_app
            .platform
            .state
            .read()
            .unwrap()
            .last_committed_block_info
            .as_ref()
            .unwrap()
            .basic_info
            .height
            + 1;

        continue_chain_for_strategy(
            abci_app,
            ChainExecutionParameters {
                block_start,
                core_height_start: 1,
                block_count: 30,
                proposers,
                quorums,
                current_quorum_hash,
                current_proposer_versions: Some(current_proposer_versions),
                current_time_ms: end_time_ms,
            },
            strategy,
            config,
            StrategyRandomness::SeedEntropy(7),
        );
    }

    #[test]
    fn run_chain_one_identity_in_solitude() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![],
            identities_inserts: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 3000,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let outcome = run_chain_for_strategy(&mut platform, 100, strategy, config, 15);

        let balance = outcome
            .abci_app
            .platform
            .drive
            .fetch_identity_balance(outcome.identities.first().unwrap().id.to_buffer(), None)
            .expect("expected to fetch balances")
            .expect("expected to have an identity to get balance from");

        assert_eq!(balance, 99864467880)
    }

    #[test]
    fn run_chain_core_height_randomly_increasing() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![],
            identities_inserts: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: 1..3,
                chance_per_block: Some(0.01),
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 3000,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();

        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        run_chain_for_strategy(&mut platform, 1000, strategy, config, 15);
    }

    #[test]
    fn run_chain_core_height_randomly_increasing_with_epoch_change() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![],
            identities_inserts: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: 1..3,
                chance_per_block: Some(0.5),
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };
        let hour_in_ms = 1000 * 60 * 60;
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 100,
            block_spacing_ms: hour_in_ms,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };

        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let outcome = run_chain_for_strategy(&mut platform, 1000, strategy, config, 15);
        assert_eq!(outcome.masternode_identity_balances.len(), 100);
        let all_have_balances = outcome
            .masternode_identity_balances
            .iter()
            .all(|(_, balance)| *balance != 0);
        assert!(all_have_balances, "all masternodes should have a balance");
    }

    #[test]
    fn run_chain_core_height_randomly_increasing_with_quorum_updates() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![],
            identities_inserts: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            total_hpmns: 500,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: 5..6,
                chance_per_block: Some(0.5),
            },
            proposer_strategy: Default::default(),
            rotate_quorums: true,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 10,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 300,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();

        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let ChainExecutionOutcome { abci_app, .. } =
            run_chain_for_strategy(&mut platform, 2000, strategy, config, 40);

        // With these params if we didn't rotate we would have at most 240
        // of the 500 hpmns that could get paid, however we are expecting that most
        // will be able to propose a block (and then get paid later on).

        let platform = abci_app.platform;
        let drive_cache = platform.drive.cache.read().unwrap();
        let counter = drive_cache
            .protocol_versions_counter
            .as_ref()
            .expect("expected a version counter");
        platform
            .drive
            .fetch_versions_with_counter(None)
            .expect("expected to get versions");

        assert_eq!(
            platform
                .state
                .read()
                .unwrap()
                .last_committed_block_info
                .as_ref()
                .unwrap()
                .basic_info
                .epoch
                .index,
            0
        );
        assert!(counter.get(&1).unwrap() > &240);
    }

    #[test]
    fn run_chain_core_height_randomly_increasing_with_quorum_updates_new_proposers() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![],
            identities_inserts: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: Some(0.2),
            },
            proposer_strategy: MasternodeListChangesStrategy {
                new_hpmns: Frequency {
                    times_per_block_range: 1..3,
                    chance_per_block: Some(0.5),
                },
                ..Default::default()
            },
            rotate_quorums: true,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 10,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 300,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();

        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let ChainExecutionOutcome { abci_app, .. } =
            run_chain_for_strategy(&mut platform, 300, strategy, config, 43);

        // With these params if we add new mns the hpmn masternode list would be 100, but we
        // can expect it to be much higher.

        let platform = abci_app.platform;
        let platform_state = platform.state.read().unwrap();

        assert!(platform_state.hpmn_masternode_list.len() > 100);
    }

    #[test]
    fn run_chain_core_height_randomly_increasing_with_quorum_updates_changing_proposers() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![],
            identities_inserts: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: Some(0.2),
            },
            proposer_strategy: MasternodeListChangesStrategy {
                new_hpmns: Frequency {
                    times_per_block_range: 1..3,
                    chance_per_block: Some(0.5),
                },
                removed_hpmns: Frequency {
                    times_per_block_range: 1..3,
                    chance_per_block: Some(0.5),
                },
                ..Default::default()
            },
            rotate_quorums: true,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 10,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 300,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();

        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let ChainExecutionOutcome { abci_app, .. } =
            run_chain_for_strategy(&mut platform, 300, strategy, config, 43);

        // With these params if we add new mns the hpmn masternode list would be randomly different than 100.

        let platform = abci_app.platform;
        let platform_state = platform.state.read().unwrap();

        assert_ne!(platform_state.hpmn_masternode_list.len(), 100);
    }

    #[test]
    fn run_chain_core_height_randomly_increasing_with_quorum_updates_updating_proposers() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![],
            identities_inserts: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: Some(0.2),
            },
            proposer_strategy: MasternodeListChangesStrategy {
                updated_hpmns: Frequency {
                    times_per_block_range: 1..3,
                    chance_per_block: Some(0.5),
                },
                ..Default::default()
            },
            rotate_quorums: true,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 10,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 300,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();

        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let ChainExecutionOutcome {
            abci_app,
            proposers,
            ..
        } = run_chain_for_strategy(&mut platform, 300, strategy, config, 43);

        // With these params if we add new mns the hpmn masternode list would be randomly different than 100.

        let platform = abci_app.platform;
        let _platform_state = platform.state.read().unwrap();

        // We need to find if any masternode has ever had their keys disabled.

        let hpmns = platform
            .drive
            .fetch_full_identities(
                proposers
                    .into_iter()
                    .map(|proposer| proposer.masternode.pro_tx_hash.into_inner())
                    .collect::<Vec<_>>()
                    .as_slice(),
                None,
            )
            .expect("expected to fetch identities");

        let has_disabled_keys = hpmns.values().any(|identity| {
            identity
                .as_ref()
                .map(|identity| {
                    identity
                        .public_keys()
                        .values()
                        .any(|key| key.disabled_at.is_some())
                })
                .unwrap_or_default()
        });
        assert!(has_disabled_keys);
    }

    #[test]
    fn run_chain_insert_one_new_identity_per_block() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![],
            identities_inserts: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: Some(QueryStrategy {
                query_identities_by_public_key_hashes: Frequency {
                    times_per_block_range: 1..5,
                    chance_per_block: None,
                },
            }),
            verify_state_transition_results: true,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 3000,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let outcome = run_chain_for_strategy(&mut platform, 100, strategy, config, 15);

        assert_eq!(outcome.identities.len(), 100);
    }

    #[test]
    fn run_chain_insert_one_new_identity_per_block_with_epoch_change() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![],
            identities_inserts: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };
        let day_in_ms = 1000 * 60 * 60 * 24;
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 100,
            block_spacing_ms: day_in_ms,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };

        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let outcome = run_chain_for_strategy(&mut platform, 150, strategy, config, 15);
        assert_eq!(outcome.identities.len(), 150);
        assert_eq!(outcome.masternode_identity_balances.len(), 100);
        let all_have_balances = outcome
            .masternode_identity_balances
            .iter()
            .all(|(_, balance)| *balance != 0);
        assert!(all_have_balances, "all masternodes should have a balance");
        assert_eq!(
            hex::encode(
                &outcome
                    .abci_app
                    .platform
                    .drive
                    .grove
                    .root_hash(None)
                    .unwrap()
                    .unwrap()
            ),
            "8e90cef4a253c8a6e2d9364f14d5b37b53a41e1f2aff9a0085c311d4e23df038".to_string()
        )
    }

    #[test]
    fn run_chain_insert_one_new_identity_and_a_contract() {
        let contract = json_document_to_created_contract(
            "tests/supporting_files/contract/dashpay/dashpay-contract-all-mutable.json",
        )
        .expect("expected to get contract from a json document");

        let strategy = Strategy {
            contracts_with_updates: vec![(contract, None)],
            operations: vec![],
            identities_inserts: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 3000,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let outcome = run_chain_for_strategy(&mut platform, 1, strategy, config, 15);

        outcome
            .abci_app
            .platform
            .drive
            .fetch_contract(
                outcome
                    .strategy
                    .contracts_with_updates
                    .first()
                    .unwrap()
                    .0
                    .data_contract
                    .id
                    .to_buffer(),
                None,
                None,
                None,
            )
            .unwrap()
            .expect("expected to execute the fetch of a contract")
            .expect("expected to get a contract");
    }

    #[test]
    fn run_chain_insert_one_new_identity_and_a_contract_with_updates() {
        let contract = json_document_to_created_contract(
            "tests/supporting_files/contract/dashpay/dashpay-contract-all-mutable.json",
        )
        .expect("expected to get contract from a json document");

        let mut contract_update_1 = json_document_to_created_contract(
            "tests/supporting_files/contract/dashpay/dashpay-contract-all-mutable-update-1.json",
        )
        .expect("expected to get contract from a json document");

        //todo: versions should start at 0 (so this should be 1)
        contract_update_1.data_contract.version = 2;

        let mut contract_update_2 = json_document_to_created_contract(
            "tests/supporting_files/contract/dashpay/dashpay-contract-all-mutable-update-2.json",
        )
        .expect("expected to get contract from a json document");

        contract_update_2.data_contract.version = 3;

        let strategy = Strategy {
            contracts_with_updates: vec![(
                contract,
                Some(BTreeMap::from([
                    (3, contract_update_1),
                    (8, contract_update_2),
                ])),
            )],
            operations: vec![],
            identities_inserts: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 3000,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let outcome = run_chain_for_strategy(&mut platform, 10, strategy, config, 15);

        outcome
            .abci_app
            .platform
            .drive
            .fetch_contract(
                outcome
                    .strategy
                    .contracts_with_updates
                    .first()
                    .unwrap()
                    .0
                    .data_contract
                    .id
                    .to_buffer(),
                None,
                None,
                None,
            )
            .unwrap()
            .expect("expected to execute the fetch of a contract")
            .expect("expected to get a contract");
    }

    #[test]
    fn run_chain_insert_one_new_identity_per_block_and_one_new_document() {
        let created_contract = json_document_to_created_contract(
            "tests/supporting_files/contract/dashpay/dashpay-contract-all-mutable.json",
        )
        .expect("expected to get contract from a json document");

        let contract = &created_contract.data_contract;

        let document_op = DocumentOp {
            contract: contract.clone(),
            action: DocumentAction::DocumentActionInsert,
            document_type: contract
                .document_type_for_name("contactRequest")
                .expect("expected a profile document type")
                .clone(),
        };

        let strategy = Strategy {
            contracts_with_updates: vec![(created_contract, None)],
            operations: vec![Operation {
                op_type: OperationType::Document(document_op),
                frequency: Frequency {
                    times_per_block_range: 1..2,
                    chance_per_block: None,
                },
            }],
            identities_inserts: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 3000,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        run_chain_for_strategy(&mut platform, 100, strategy, config, 15);
    }

    #[test]
    fn run_chain_insert_one_new_identity_per_block_and_a_document_with_epoch_change() {
        let created_contract = json_document_to_created_contract(
            "tests/supporting_files/contract/dashpay/dashpay-contract-all-mutable.json",
        )
        .expect("expected to get contract from a json document");

        let contract = &created_contract.data_contract;

        let document_op = DocumentOp {
            contract: contract.clone(),
            action: DocumentAction::DocumentActionInsert,
            document_type: contract
                .document_type_for_name("contactRequest")
                .expect("expected a profile document type")
                .clone(),
        };

        let strategy = Strategy {
            contracts_with_updates: vec![(created_contract, None)],
            operations: vec![Operation {
                op_type: OperationType::Document(document_op),
                frequency: Frequency {
                    times_per_block_range: 1..2,
                    chance_per_block: None,
                },
            }],
            identities_inserts: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };
        let day_in_ms = 1000 * 60 * 60 * 24;
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 100,
            block_spacing_ms: day_in_ms,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let block_count = 120;
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let outcome = run_chain_for_strategy(&mut platform, block_count, strategy, config, 15);
        assert_eq!(outcome.identities.len() as u64, block_count);
        assert_eq!(outcome.masternode_identity_balances.len(), 100);
        let all_have_balances = outcome
            .masternode_identity_balances
            .iter()
            .all(|(_, balance)| *balance != 0);
        assert!(all_have_balances, "all masternodes should have a balance");
    }

    #[test]
    fn run_chain_insert_one_new_identity_per_block_document_insertions_and_deletions_with_epoch_change(
    ) {
        let created_contract = json_document_to_created_contract(
            "tests/supporting_files/contract/dashpay/dashpay-contract-all-mutable.json",
        )
        .expect("expected to get contract from a json document");

        let contract = &created_contract.data_contract;

        let document_insertion_op = DocumentOp {
            contract: contract.clone(),
            action: DocumentAction::DocumentActionInsert,
            document_type: contract
                .document_type_for_name("contactRequest")
                .expect("expected a profile document type")
                .clone(),
        };

        let document_deletion_op = DocumentOp {
            contract: contract.clone(),
            action: DocumentAction::DocumentActionDelete,
            document_type: contract
                .document_type_for_name("contactRequest")
                .expect("expected a profile document type")
                .clone(),
        };

        let strategy = Strategy {
            contracts_with_updates: vec![(created_contract, None)],
            operations: vec![
                Operation {
                    op_type: OperationType::Document(document_insertion_op),
                    frequency: Frequency {
                        times_per_block_range: 1..2,
                        chance_per_block: None,
                    },
                },
                Operation {
                    op_type: OperationType::Document(document_deletion_op),
                    frequency: Frequency {
                        times_per_block_range: 1..2,
                        chance_per_block: None,
                    },
                },
            ],
            identities_inserts: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };
        let day_in_ms = 1000 * 60 * 60 * 24;
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 100,
            block_spacing_ms: day_in_ms,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let block_count = 120;
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let outcome = run_chain_for_strategy(&mut platform, block_count, strategy, config, 15);
        assert_eq!(outcome.identities.len() as u64, block_count);
        assert_eq!(outcome.masternode_identity_balances.len(), 100);
        let all_have_balances = outcome
            .masternode_identity_balances
            .iter()
            .all(|(_, balance)| *balance != 0);
        assert!(all_have_balances, "all masternodes should have a balance");
    }

    #[test]
    fn run_chain_insert_one_new_identity_per_block_many_document_insertions_and_deletions_with_epoch_change(
    ) {
        let created_contract = json_document_to_created_contract(
            "tests/supporting_files/contract/dashpay/dashpay-contract-all-mutable.json",
        )
        .expect("expected to get contract from a json document");

        let contract = &created_contract.data_contract;

        let document_insertion_op = DocumentOp {
            contract: contract.clone(),
            action: DocumentAction::DocumentActionInsert,
            document_type: contract
                .document_type_for_name("contactRequest")
                .expect("expected a profile document type")
                .clone(),
        };

        let document_deletion_op = DocumentOp {
            contract: contract.clone(),
            action: DocumentAction::DocumentActionDelete,
            document_type: contract
                .document_type_for_name("contactRequest")
                .expect("expected a profile document type")
                .clone(),
        };

        let strategy = Strategy {
            contracts_with_updates: vec![(created_contract, None)],
            operations: vec![
                Operation {
                    op_type: OperationType::Document(document_insertion_op),
                    frequency: Frequency {
                        times_per_block_range: 1..10,
                        chance_per_block: None,
                    },
                },
                Operation {
                    op_type: OperationType::Document(document_deletion_op),
                    frequency: Frequency {
                        times_per_block_range: 1..10,
                        chance_per_block: None,
                    },
                },
            ],
            identities_inserts: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };
        let day_in_ms = 1000 * 60 * 60 * 24;
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 100,
            block_spacing_ms: day_in_ms,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };

        let block_count = 120;
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let outcome = run_chain_for_strategy(&mut platform, block_count, strategy, config, 15);
        assert_eq!(outcome.identities.len() as u64, block_count);
        assert_eq!(outcome.masternode_identity_balances.len(), 100);
        let all_have_balances = outcome
            .masternode_identity_balances
            .iter()
            .all(|(_, balance)| *balance != 0);
        assert!(all_have_balances, "all masternodes should have a balance");
        assert_eq!(
            hex::encode(
                &outcome
                    .abci_app
                    .platform
                    .drive
                    .grove
                    .root_hash(None)
                    .unwrap()
                    .unwrap()
            ),
            "0becd3a88fff5f1e4755ecebe94ab7aab1ac59a536e5feb8db6cb1b224855915".to_string()
        )
    }

    #[test]
    fn run_chain_insert_many_new_identity_per_block_many_document_insertions_and_deletions_with_epoch_change(
    ) {
        let created_contract = json_document_to_created_contract(
            "tests/supporting_files/contract/dashpay/dashpay-contract-all-mutable.json",
        )
        .expect("expected to get contract from a json document");

        let contract = &created_contract.data_contract;

        let document_insertion_op = DocumentOp {
            contract: contract.clone(),
            action: DocumentAction::DocumentActionInsert,
            document_type: contract
                .document_type_for_name("contactRequest")
                .expect("expected a profile document type")
                .clone(),
        };

        let document_deletion_op = DocumentOp {
            contract: contract.clone(),
            action: DocumentAction::DocumentActionDelete,
            document_type: contract
                .document_type_for_name("contactRequest")
                .expect("expected a profile document type")
                .clone(),
        };

        let strategy = Strategy {
            contracts_with_updates: vec![(created_contract, None)],
            operations: vec![
                Operation {
                    op_type: OperationType::Document(document_insertion_op),
                    frequency: Frequency {
                        times_per_block_range: 1..40,
                        chance_per_block: None,
                    },
                },
                Operation {
                    op_type: OperationType::Document(document_deletion_op),
                    frequency: Frequency {
                        times_per_block_range: 1..15,
                        chance_per_block: None,
                    },
                },
            ],
            identities_inserts: Frequency {
                times_per_block_range: 1..30,
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };

        let day_in_ms = 1000 * 60 * 60 * 24;

        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 100,
            block_spacing_ms: day_in_ms,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let block_count = 30;
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let outcome = run_chain_for_strategy(&mut platform, block_count, strategy, config, 15);
        assert_eq!(outcome.identities.len() as u64, 449);
        assert_eq!(outcome.masternode_identity_balances.len(), 100);
        let balance_count = outcome
            .masternode_identity_balances
            .into_iter()
            .filter(|(_, balance)| *balance != 0)
            .count();
        assert_eq!(balance_count, 19); // 1 epoch worth of proposers
    }

    #[test]
    fn run_chain_insert_many_new_identity_per_block_many_document_insertions_updates_and_deletions_with_epoch_change(
    ) {
        let created_contract = json_document_to_created_contract(
            "tests/supporting_files/contract/dashpay/dashpay-contract-all-mutable.json",
        )
        .expect("expected to get contract from a json document");

        let contract = &created_contract.data_contract;

        let document_insertion_op = DocumentOp {
            contract: contract.clone(),
            action: DocumentAction::DocumentActionInsert,
            document_type: contract
                .document_type_for_name("contactRequest")
                .expect("expected a profile document type")
                .clone(),
        };

        let document_replace_op = DocumentOp {
            contract: contract.clone(),
            action: DocumentActionReplace,
            document_type: contract
                .document_type_for_name("contactRequest")
                .expect("expected a profile document type")
                .clone(),
        };

        let document_deletion_op = DocumentOp {
            contract: contract.clone(),
            action: DocumentAction::DocumentActionDelete,
            document_type: contract
                .document_type_for_name("contactRequest")
                .expect("expected a profile document type")
                .clone(),
        };

        let strategy = Strategy {
            contracts_with_updates: vec![(created_contract, None)],
            operations: vec![
                Operation {
                    op_type: OperationType::Document(document_insertion_op),
                    frequency: Frequency {
                        times_per_block_range: 1..40,
                        chance_per_block: None,
                    },
                },
                Operation {
                    op_type: OperationType::Document(document_replace_op),
                    frequency: Frequency {
                        times_per_block_range: 1..5,
                        chance_per_block: None,
                    },
                },
                Operation {
                    op_type: OperationType::Document(document_deletion_op),
                    frequency: Frequency {
                        times_per_block_range: 1..5,
                        chance_per_block: None,
                    },
                },
            ],
            identities_inserts: Frequency {
                times_per_block_range: 1..6,
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };

        let day_in_ms = 1000 * 60 * 60 * 24;

        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 100,
            block_spacing_ms: day_in_ms,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let block_count = 30;
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let outcome = run_chain_for_strategy(&mut platform, block_count, strategy, config, 15);
        assert_eq!(outcome.identities.len() as u64, 97);
        assert_eq!(outcome.masternode_identity_balances.len(), 100);
        let balance_count = outcome
            .masternode_identity_balances
            .into_iter()
            .filter(|(_, balance)| *balance != 0)
            .count();
        assert_eq!(balance_count, 19); // 1 epoch worth of proposers
    }

    #[test]
    fn run_chain_top_up_identities() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![Operation {
                op_type: OperationType::IdentityTopUp,
                frequency: Frequency {
                    times_per_block_range: 1..3,
                    chance_per_block: None,
                },
            }],
            identities_inserts: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: true,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 3000,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let outcome = run_chain_for_strategy(&mut platform, 10, strategy, config, 15);

        let max_initial_balance = 100000000000u64; // TODO: some centralized way for random test data (`arbitrary` maybe?)
        let balances = outcome
            .abci_app
            .platform
            .drive
            .fetch_identities_balances(
                &outcome
                    .identities
                    .into_iter()
                    .map(|identity| identity.id.to_buffer())
                    .collect(),
                None,
            )
            .expect("expected to fetch balances");

        assert!(balances
            .into_iter()
            .any(|(_, balance)| balance > max_initial_balance));
    }

    #[test]
    fn run_chain_update_identities_add_keys() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![Operation {
                op_type: OperationType::IdentityUpdate(IdentityUpdateOp::IdentityUpdateAddKeys(3)),
                frequency: Frequency {
                    times_per_block_range: 1..2,
                    chance_per_block: None,
                },
            }],
            identities_inserts: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            // because we can add an identity and add keys to it in the same block
            // the result would be different then expected
            verify_state_transition_results: false,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 3000,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let outcome = run_chain_for_strategy(&mut platform, 10, strategy, config, 15);

        let identities = outcome
            .abci_app
            .platform
            .drive
            .fetch_full_identities(
                outcome
                    .identities
                    .into_iter()
                    .map(|identity| identity.id.to_buffer())
                    .collect::<Vec<_>>()
                    .as_slice(),
                None,
            )
            .expect("expected to fetch balances");

        assert!(identities
            .into_iter()
            .any(|(_, identity)| { identity.expect("expected identity").public_keys.len() > 7 }));
    }

    #[test]
    fn run_chain_update_identities_remove_keys() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![Operation {
                op_type: OperationType::IdentityUpdate(IdentityUpdateOp::IdentityUpdateDisableKey(
                    3,
                )),
                frequency: Frequency {
                    times_per_block_range: 1..2,
                    chance_per_block: None,
                },
            }],
            identities_inserts: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            // because we can add an identity and remove keys to it in the same block
            // the result would be different then expected
            verify_state_transition_results: false,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 3000,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let outcome = run_chain_for_strategy(&mut platform, 10, strategy, config, 15);

        let identities = outcome
            .abci_app
            .platform
            .drive
            .fetch_full_identities(
                outcome
                    .identities
                    .into_iter()
                    .map(|identity| identity.id.to_buffer())
                    .collect::<Vec<_>>()
                    .as_slice(),
                None,
            )
            .expect("expected to fetch balances");

        assert!(identities.into_iter().any(|(_, identity)| {
            identity
                .expect("expected identity")
                .public_keys()
                .into_iter()
                .any(|(_, public_key)| public_key.is_disabled())
        }));
    }

    #[test]
    fn run_chain_top_up_and_withdraw_from_identities() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![
                Operation {
                    op_type: OperationType::IdentityTopUp,
                    frequency: Frequency {
                        times_per_block_range: 1..4,
                        chance_per_block: None,
                    },
                },
                Operation {
                    op_type: OperationType::IdentityWithdrawal,
                    frequency: Frequency {
                        times_per_block_range: 1..4,
                        chance_per_block: None,
                    },
                },
            ],
            identities_inserts: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            total_hpmns: 100,
            extra_normal_mns: 0,
            quorum_count: 24,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            // because we can add an identity and withdraw from it in the same block
            // the result would be different then expected
            verify_state_transition_results: false,
        };
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 100,
            validator_set_quorum_rotation_block_count: 25,
            block_spacing_ms: 3000,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        let outcome = run_chain_for_strategy(&mut platform, 10, strategy, config, 15);

        assert_eq!(outcome.identities.len(), 10);
        assert_eq!(outcome.withdrawals.len(), 14);
    }

    #[test]
    fn run_chain_rotation_is_deterministic_1_block() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![],
            identities_inserts: Frequency {
                //we do this to create some paying transactions
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            total_hpmns: 50,
            extra_normal_mns: 0,
            quorum_count: 10,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: true,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: false,
        };
        let day_in_ms = 1000 * 60 * 60 * 24;
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 3,
            validator_set_quorum_rotation_block_count: 1,
            block_spacing_ms: day_in_ms,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };

        let mut platforms = Vec::new();
        let mut outcomes = Vec::new();

        for _ in 0..2 {
            let platform = TestPlatformBuilder::new()
                .with_config(config.clone())
                .build_with_mock_rpc();
            platforms.push(platform);
        }

        for platform in &mut platforms {
            platform
                .core_rpc
                .expect_get_best_chain_lock()
                .returning(move || {
                    Ok(CoreChainLock {
                        core_block_height: 10,
                        core_block_hash: [1; 32].to_vec(),
                        signature: [2; 96].to_vec(),
                    })
                });

            let outcome = run_chain_for_strategy(platform, 1, strategy.clone(), config.clone(), 7);
            outcomes.push(outcome);
        }

        let first_proposers_fingerprint = hash_to_hex_string(
            outcomes[0]
                .proposers
                .iter()
                .map(|masternode_list_item_with_updates| {
                    hex::encode(masternode_list_item_with_updates.masternode.pro_tx_hash)
                })
                .join("|"),
        );

        assert!(outcomes.iter().all(|outcome| {
            let last_proposers_fingerprint = hash_to_hex_string(
                outcome
                    .proposers
                    .iter()
                    .map(|masternode_list_item_with_updates| {
                        hex::encode(masternode_list_item_with_updates.masternode.pro_tx_hash)
                    })
                    .join("|"),
            );

            first_proposers_fingerprint == last_proposers_fingerprint
        }));

        let first_masternodes_fingerprint = hash_to_hex_string(
            outcomes[0]
                .masternode_identity_balances
                .keys()
                .map(|pro_tx_hash| hex::encode(pro_tx_hash))
                .join("|"),
        );

        assert!(outcomes.iter().all(|outcome| {
            let last_masternodes_fingerprint = hash_to_hex_string(
                outcome
                    .masternode_identity_balances
                    .keys()
                    .map(|pro_tx_hash| hex::encode(pro_tx_hash))
                    .join("|"),
            );

            first_masternodes_fingerprint == last_masternodes_fingerprint
        }));

        let first_validator_set_fingerprint = hash_to_hex_string(
            outcomes[0]
                .current_quorum()
                .validator_set
                .iter()
                .map(|validator| hex::encode(validator.pro_tx_hash))
                .join("|"),
        );

        assert!(outcomes.iter().all(|outcome| {
            let last_validator_set_fingerprint = hash_to_hex_string(
                outcome
                    .current_quorum()
                    .validator_set
                    .iter()
                    .map(|validator| hex::encode(validator.pro_tx_hash))
                    .join("|"),
            );

            first_validator_set_fingerprint == last_validator_set_fingerprint
        }));

        let first_last_app_hash = outcomes[0]
            .abci_app
            .platform
            .drive
            .grove
            .root_hash(None)
            .unwrap()
            .expect("should return app hash");

        assert!(outcomes.iter().all(|outcome| {
            let last_app_hash = outcome
                .abci_app
                .platform
                .drive
                .grove
                .root_hash(None)
                .unwrap()
                .expect("should return app hash");

            last_app_hash == first_last_app_hash
        }));
    }

    #[test]
    fn run_chain_heavy_rotation_deterministic_before_payout() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![],
            identities_inserts: Frequency {
                //we do this to create some paying transactions
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            total_hpmns: 500,
            extra_normal_mns: 0,
            quorum_count: 100,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: true,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: false,
        };
        let day_in_ms = 1000 * 60 * 60 * 24;
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 3,
            validator_set_quorum_rotation_block_count: 1,
            block_spacing_ms: day_in_ms,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform_a = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        let mut platform_b = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform_a
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        platform_b
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });

        let outcome_a =
            run_chain_for_strategy(&mut platform_a, 18, strategy.clone(), config.clone(), 7);
        let outcome_b = run_chain_for_strategy(&mut platform_b, 18, strategy, config, 7);
        assert_eq!(outcome_a.end_epoch_index, outcome_b.end_epoch_index); // 100/18
        assert_eq!(outcome_a.masternode_identity_balances.len(), 500); // 500 nodes
        assert_eq!(outcome_b.masternode_identity_balances.len(), 500); // 500 nodes
        assert_eq!(outcome_a.end_epoch_index, 0); // 100/18
        let masternodes_fingerprint_a = hash_to_hex_string(
            outcome_a
                .masternode_identity_balances
                .keys()
                .map(|pro_tx_hash| hex::encode(pro_tx_hash))
                .join("|"),
        );
        assert_eq!(
            masternodes_fingerprint_a,
            "3f13e499c5c49b04ab1edf2bbddca733fd9cf6f92875ccf51e827a6f4bf044e8".to_string()
        );
        let masternodes_fingerprint_b = hash_to_hex_string(
            outcome_b
                .masternode_identity_balances
                .keys()
                .map(|pro_tx_hash| hex::encode(pro_tx_hash))
                .join("|"),
        );
        assert_eq!(
            masternodes_fingerprint_b,
            "3f13e499c5c49b04ab1edf2bbddca733fd9cf6f92875ccf51e827a6f4bf044e8".to_string()
        );

        let last_app_hash_a = outcome_a
            .abci_app
            .platform
            .drive
            .grove
            .root_hash(None)
            .unwrap()
            .expect("should return app hash");

        let last_app_hash_b = outcome_b
            .abci_app
            .platform
            .drive
            .grove
            .root_hash(None)
            .unwrap()
            .expect("should return app hash");

        assert_eq!(last_app_hash_a, last_app_hash_b);

        let balance_count = outcome_a
            .masternode_identity_balances
            .into_iter()
            .filter(|(_, balance)| *balance != 0)
            .count();
        assert_eq!(balance_count, 0);
    }

    #[test]
    fn run_chain_proposer_proposes_a_chainlock_that_would_remove_themselves_from_the_list_deterministic(
    ) {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![],
            identities_inserts: Frequency {
                //we do this to create some paying transactions
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            total_hpmns: 500,
            extra_normal_mns: 0,
            quorum_count: 100,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: 1..2,
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: true,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: false,
        };
        let day_in_ms = 1000 * 60 * 60 * 24;
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 3,
            validator_set_quorum_rotation_block_count: 1,
            block_spacing_ms: day_in_ms,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let mut platform_a = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        let mut platform_b = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();
        platform_a
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });
        platform_b
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });

        let outcome_a =
            run_chain_for_strategy(&mut platform_a, 100, strategy.clone(), config.clone(), 7);
        let outcome_b = run_chain_for_strategy(&mut platform_b, 100, strategy, config, 7);
        assert_eq!(outcome_a.end_epoch_index, outcome_b.end_epoch_index); // 100/18
        assert_eq!(outcome_a.masternode_identity_balances.len(), 500); // 500 nodes
        assert_eq!(outcome_b.masternode_identity_balances.len(), 500); // 500 nodes
                                                                       //assert_eq!(outcome_a.end_epoch_index, 1); // 100/18
        let masternodes_fingerprint_a = hash_to_hex_string(
            outcome_a
                .masternode_identity_balances
                .keys()
                .map(|pro_tx_hash| hex::encode(pro_tx_hash))
                .join("|"),
        );
        assert_eq!(
            masternodes_fingerprint_a,
            "3f13e499c5c49b04ab1edf2bbddca733fd9cf6f92875ccf51e827a6f4bf044e8".to_string()
        );
        let masternodes_fingerprint_b = hash_to_hex_string(
            outcome_b
                .masternode_identity_balances
                .keys()
                .map(|pro_tx_hash| hex::encode(pro_tx_hash))
                .join("|"),
        );
        assert_eq!(
            masternodes_fingerprint_b,
            "3f13e499c5c49b04ab1edf2bbddca733fd9cf6f92875ccf51e827a6f4bf044e8".to_string()
        );

        let last_app_hash_a = outcome_a
            .abci_app
            .platform
            .drive
            .grove
            .root_hash(None)
            .unwrap()
            .expect("should return app hash");

        let last_app_hash_b = outcome_b
            .abci_app
            .platform
            .drive
            .grove
            .root_hash(None)
            .unwrap()
            .expect("should return app hash");

        assert_eq!(last_app_hash_a, last_app_hash_b);

        let balance_count = outcome_a
            .masternode_identity_balances
            .into_iter()
            .filter(|(_, balance)| *balance != 0)
            .count();
        // we have a maximum 90 quorums, that could have been used, 4 were used twice
        assert_eq!(balance_count, 86);
    }

    #[test]
    fn run_chain_stop_and_restart_with_rotation() {
        let strategy = Strategy {
            contracts_with_updates: vec![],
            operations: vec![],
            identities_inserts: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            total_hpmns: 500,
            extra_normal_mns: 0,
            quorum_count: 100,
            upgrading_info: None,
            core_height_increase: Frequency {
                times_per_block_range: Default::default(),
                chance_per_block: None,
            },
            proposer_strategy: Default::default(),
            rotate_quorums: false,
            failure_testing: None,
            query_testing: None,
            verify_state_transition_results: false,
        };
        let day_in_ms = 1000 * 60 * 60 * 24;
        let config = PlatformConfig {
            verify_sum_trees: true,
            quorum_size: 3,
            validator_set_quorum_rotation_block_count: 1,
            block_spacing_ms: day_in_ms,
            testing_configs: PlatformTestConfig::default_with_no_block_signing(),
            ..Default::default()
        };
        let TempPlatform {
            mut platform,
            tempdir: _,
        } = TestPlatformBuilder::new()
            .with_config(config.clone())
            .build_with_mock_rpc();

        platform
            .core_rpc
            .expect_get_best_chain_lock()
            .returning(move || {
                Ok(CoreChainLock {
                    core_block_height: 10,
                    core_block_hash: [1; 32].to_vec(),
                    signature: [2; 96].to_vec(),
                })
            });

        let ChainExecutionOutcome {
            abci_app,
            proposers,
            quorums,
            current_quorum_hash,
            current_proposer_versions,
            end_time_ms,
            ..
        } = run_chain_for_strategy(&mut platform, 100, strategy.clone(), config.clone(), 89);

        let known_root_hash = abci_app
            .platform
            .drive
            .grove
            .root_hash(None)
            .unwrap()
            .expect("expected root hash");

        abci_app
            .platform
            .recreate_state()
            .expect("expected to recreate state");

        let ResponseInfo {
            data: _,
            version: _,
            app_version: _,
            last_block_height,
            last_block_app_hash,
        } = abci_app
            .info(RequestInfo {
                version: "0.12.0".to_string(),
                block_version: 0,
                p2p_version: 0,
                abci_version: "0.20.0".to_string(),
            })
            .expect("expected to call info");

        assert_eq!(last_block_height, 100);
        assert_eq!(last_block_app_hash, known_root_hash);

        let block_start = abci_app
            .platform
            .state
            .read()
            .unwrap()
            .last_committed_block_info
            .as_ref()
            .unwrap()
            .basic_info
            .height
            + 1;

        continue_chain_for_strategy(
            abci_app,
            ChainExecutionParameters {
                block_start,
                core_height_start: 10,
                block_count: 30,
                proposers,
                quorums,
                current_quorum_hash,
                current_proposer_versions: Some(current_proposer_versions),
                current_time_ms: end_time_ms,
            },
            strategy,
            config,
            StrategyRandomness::SeedEntropy(block_start),
        );
    }
}
