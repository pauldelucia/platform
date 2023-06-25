mod v0;

use grovedb::operations::insert::InsertOptions;
use grovedb::{Element, TransactionArg};
use path::SubtreePath;
use dpp::version::drive_versions::DriveVersion;
use crate::drive::Drive;
use crate::error::drive::DriveError;
use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;

impl Drive {
    /// Inserts an empty sum tree into groveDB at the specified path and key.
    /// The operation's cost is then added to `drive_operations` for later processing.
    ///
    /// # Parameters
    /// * `path`: The groveDB hierarchical authenticated structure path where the new element is to be inserted.
    /// * `key`: The key where the new element should be inserted in the subtree.
    /// * `transaction`: The groveDB transaction associated with this operation.
    /// * `options`: Optional insert options to further configure the operation.
    /// * `drive_operations`: A vector to collect the costs of operations for later computation.
    /// * `platform_version`: The platform version to select the correct function version to run.
    ///
    /// # Returns
    /// * `Ok(())` if the operation was successful.
    /// * `Err(DriveError::UnknownVersionMismatch)` if the platform version does not match known versions.
    pub fn grove_insert_empty_sum_tree<B: AsRef<[u8]>>(
        &self,
        path: SubtreePath<'_, B>,
        key: &[u8],
        transaction: TransactionArg,
        options: Option<InsertOptions>,
        drive_operations: &mut Vec<LowLevelDriveOperation>,
        drive_version: &DriveVersion,
    ) -> Result<(), Error> {
        match drive_version.grove_methods.basic.grove_insert_empty_sum_tree {
            0 => self.grove_insert_empty_sum_tree_v0(path, key, transaction, options, drive_operations),
            version => Err(Error::Drive(DriveError::UnknownVersionMismatch {
                method: "grove_insert_empty_sum_tree".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }
}