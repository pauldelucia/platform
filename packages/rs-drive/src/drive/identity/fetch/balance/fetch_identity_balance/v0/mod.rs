use crate::drive::balances::balance_path;

use crate::drive::grove_operations::DirectQueryType;
use crate::drive::grove_operations::QueryTarget::QueryTargetValue;
use crate::drive::Drive;
use crate::error::drive::DriveError;
use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;
use dpp::block::block_info::BlockInfo;
use dpp::fee::fee_result::FeeResult;
use dpp::fee::Credits;

use dpp::version::PlatformVersion;
use grovedb::Element::SumItem;
use grovedb::TransactionArg;

impl Drive {
    /// Fetches the Identity's balance from the backing store
    /// Passing apply as false get the estimated cost instead
    pub(super) fn fetch_identity_balance_v0(
        &self,
        identity_id: [u8; 32],
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<Option<Credits>, Error> {
        let mut drive_operations: Vec<LowLevelDriveOperation> = vec![];
        self.fetch_identity_balance_operations_v0(
            identity_id,
            true,
            transaction,
            &mut drive_operations,
            platform_version,
        )
    }

    /// Fetches the Identity's balance from the backing store
    /// Passing apply as false get the estimated cost instead
    pub(super) fn fetch_identity_balance_with_costs_v0(
        &self,
        identity_id: [u8; 32],
        block_info: &BlockInfo,
        apply: bool,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<(Option<Credits>, FeeResult), Error> {
        let mut drive_operations: Vec<LowLevelDriveOperation> = vec![];
        let value = self.fetch_identity_balance_operations_v0(
            identity_id,
            apply,
            transaction,
            &mut drive_operations,
            platform_version,
        )?;
        let fees = Drive::calculate_fee(
            None,
            Some(drive_operations),
            &block_info.epoch,
            platform_version,
        )?;
        Ok((value, fees))
    }

    /// Creates the operations to get Identity's balance from the backing store
    /// This gets operations based on apply flag (stateful vs stateless)
    pub(super) fn fetch_identity_balance_operations_v0(
        &self,
        identity_id: [u8; 32],
        apply: bool,
        transaction: TransactionArg,
        drive_operations: &mut Vec<LowLevelDriveOperation>,
        platform_version: &PlatformVersion,
    ) -> Result<Option<Credits>, Error> {
        let direct_query_type = if apply {
            DirectQueryType::StatefulDirectQuery
        } else {
            // 8 is the size of a i64 used in sum trees
            DirectQueryType::StatelessDirectQuery {
                in_tree_using_sums: true,
                query_target: QueryTargetValue(8),
            }
        };

        let balance_path = balance_path();

        match self.grove_get_raw(
            (&balance_path).into(),
            identity_id.as_slice(),
            direct_query_type,
            transaction,
            drive_operations,
            &platform_version.drive,
        ) {
            Ok(Some(SumItem(balance, _))) if balance >= 0 => Ok(Some(balance as Credits)),

            Ok(None) | Err(Error::GroveDB(grovedb::Error::PathKeyNotFound(_))) => {
                if apply {
                    Ok(None)
                } else {
                    Ok(Some(0))
                }
            }

            Ok(Some(SumItem(..))) => Err(Error::Drive(DriveError::CorruptedElementType(
                "identity balance was present but was negative",
            ))),

            Ok(Some(_)) => Err(Error::Drive(DriveError::CorruptedElementType(
                "identity balance was present but was not identified as a sum item",
            ))),

            Err(e) => Err(e),
        }
    }
}
