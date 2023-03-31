use dpp::validation::SimpleValidationResult;
use crate::abci::handlers::TenderdashAbci;
use crate::abci::messages::{
    AfterFinalizeBlockRequest, BlockBeginRequest, BlockEndRequest, BlockFees,
};
use crate::block::BlockExecutionContext;
use crate::error::execution::ExecutionError;
use crate::error::Error;
use crate::execution::execution_event::{ExecutionEvent, ExecutionResult};
use crate::platform::Platform;
use crate::rpc::core::CoreRPCLike;
use crate::state::PlatformState;
use drive::dpp::identity::PartialIdentity;
use drive::dpp::util::deserializer::ProtocolVersion;
use drive::drive::batch::DriveOperation;
use drive::drive::block_info::BlockInfo;
use drive::drive::Drive;
use drive::error::Error::GroveDB;
use drive::fee::result::FeeResult;
use drive::grovedb::Transaction;
use crate::execution::execution_event::ExecutionResult::{ConsensusExecutionError, SuccessfulFreeExecution, SuccessfulPaidExecution};


impl<'a, C> Platform<'a, C>
where
    C: CoreRPCLike,
{
    pub(crate) fn execute_event(
        &self,
        event: ExecutionEvent,
        block_info: &BlockInfo,
        transaction: &Transaction,
    ) -> Result<ExecutionResult, Error> {
        //todo: we need to split out errors
        //  between failed execution and internal errors
        match event {
            ExecutionEvent::PaidDriveEvent {
                identity,
                verify_balance_with_dry_run,
                operations,
            } => {
                let balance = identity.balance.ok_or(Error::Execution(
                    ExecutionError::CorruptedCodeExecution(
                        "partial identity info with no balance",
                    ),
                ))?;
                let enough_balance = if verify_balance_with_dry_run {
                    let estimated_fee_result = self
                        .drive
                        .apply_drive_operations(
                            operations.clone(),
                            false,
                            block_info,
                            Some(transaction),
                        )
                        .map_err(Error::Drive)?;

                    // TODO: Should take into account refunds as well
                    balance >= estimated_fee_result.total_base_fee()
                } else {
                    true
                };

                if enough_balance {
                    let individual_fee_result = self
                        .drive
                        .apply_drive_operations(operations, true, block_info, Some(transaction))
                        .map_err(Error::Drive)?;

                    let balance_change =
                        individual_fee_result.into_balance_change(identity.id.to_buffer());

                    let outcome = self.drive.apply_balance_change_from_fee_to_identity(
                        balance_change.clone(),
                        Some(transaction),
                    )?;

                    // println!("State transition fees {:#?}", outcome.actual_fee_paid);
                    //
                    // println!(
                    //     "Identity balance {:?} changed {:#?}",
                    //     identity.balance,
                    //     balance_change.change()
                    // );

                    Ok(SuccessfulPaidExecution(outcome.actual_fee_paid))
                } else {
                    Ok(ConsensusExecutionError(SimpleValidationResult::new_with_errors()))
                }
            }
            ExecutionEvent::FreeDriveEvent { operations } => {
                self.drive
                    .apply_drive_operations(operations, true, block_info, Some(transaction))
                    .map_err(Error::Drive)?;
                Ok(SuccessfulFreeExecution)
            }
        }
    }

    fn run_events(
        &self,
        events: Vec<ExecutionEvent>,
        block_info: &BlockInfo,
        transaction: &Transaction,
    ) -> Result<FeeResult, Error> {
        let mut total_fees = FeeResult::default();
        for event in events {
            match event {
                ExecutionEvent::PaidDriveEvent {
                    identity,
                    verify_balance_with_dry_run,
                    operations,
                } => {
                    let balance = identity.balance.ok_or(Error::Execution(
                        ExecutionError::CorruptedCodeExecution(
                            "partial identity info with no balance",
                        ),
                    ))?;
                    let enough_balance = if verify_balance_with_dry_run {
                        let estimated_fee_result = self
                            .drive
                            .apply_drive_operations(
                                operations.clone(),
                                false,
                                block_info,
                                Some(transaction),
                            )
                            .map_err(Error::Drive)?;

                        // TODO: Should take into account refunds as well
                        balance >= estimated_fee_result.total_base_fee()
                    } else {
                        true
                    };

                    if enough_balance {
                        let individual_fee_result = self
                            .drive
                            .apply_drive_operations(operations, true, block_info, Some(transaction))
                            .map_err(Error::Drive)?;

                        let balance_change =
                            individual_fee_result.into_balance_change(identity.id.to_buffer());

                        let outcome = self.drive.apply_balance_change_from_fee_to_identity(
                            balance_change.clone(),
                            Some(transaction),
                        )?;

                        // println!("State transition fees {:#?}", outcome.actual_fee_paid);
                        //
                        // println!(
                        //     "Identity balance {:?} changed {:#?}",
                        //     identity.balance,
                        //     balance_change.change()
                        // );

                        total_fees
                            .checked_add_assign(outcome.actual_fee_paid)
                            .map_err(Error::Drive)?;
                    }
                }
                ExecutionEvent::FreeDriveEvent { operations } => {
                    self.drive
                        .apply_drive_operations(operations, true, block_info, Some(transaction))
                        .map_err(Error::Drive)?;
                }
            }
        }
        Ok(total_fees)
    }

    /// Execute a block with various state transitions
    pub fn execute_block(
        &self,
        proposer: [u8; 32],
        proposed_version: ProtocolVersion,
        total_hpmns: u32,
        block_info: BlockInfo,
        state_transitions: Vec<ExecutionEvent>,
    ) -> Result<(), Error> {
        let transaction = self.drive.grove.start_transaction();
        // Processing block
        let block_begin_request = BlockBeginRequest {
            block_height: block_info.height,
            block_time_ms: block_info.time_ms,
            previous_block_time_ms: self
                .state
                .read()
                .unwrap()
                .last_block_info
                .as_ref()
                .map(|block_info| block_info.time_ms),
            proposer_pro_tx_hash: proposer,
            proposed_app_version: proposed_version,
            validator_set_quorum_hash: Default::default(),
            last_synced_core_height: 1,
            core_chain_locked_height: 1,
            total_hpmns,
        };

        // println!("Block #{}", block_info.height);

        let _block_begin_response = self.block_begin(block_begin_request).unwrap_or_else(|e| {
            panic!(
                "should begin process block #{} at time #{} : {e}",
                block_info.height, block_info.time_ms
            )
        });

        // println!("{:#?}", block_begin_response);

        let total_fees = self.run_events(state_transitions, &block_info, &transaction)?;

        let fees = BlockFees::from_fee_result(total_fees);

        let block_end_request = BlockEndRequest { fees };

        let _block_end_response = self.block_end(block_end_request).unwrap_or_else(|e| {
            panic!(
                "engine should end process block #{} at time #{} : {}",
                block_info.height, block_info.time_ms, e
            )
        });

        // println!("{:#?}", block_end_response);

        let after_finalize_block_request = AfterFinalizeBlockRequest {
            updated_data_contract_ids: Vec::new(),
        };

        self.after_finalize_block(after_finalize_block_request)
            .unwrap_or_else(|_| {
                panic!(
                    "should begin process block #{} at time #{}",
                    block_info.height, block_info.time_ms
                )
            });

        // TODO: Move to `after_finalize_block` so it will be called by JS Drive too
        self.state.write().unwrap().last_block_info = Some(block_info.clone());

        Ok(())
    }
}
