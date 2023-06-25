use grovedb::batch::GroveDbOp;
use grovedb::TransactionArg;
use crate::drive::batch::GroveDbOpBatch;
use crate::drive::Drive;
use crate::error::Error;

impl Drive {
    /// Applies the given groveDB operation
    pub(super) fn grove_apply_operation_v0(
        &self,
        operation: GroveDbOp,
        validate: bool,
        transaction: TransactionArg,
    ) -> Result<(), Error> {
        self.grove_apply_batch_with_add_costs(
            GroveDbOpBatch {
                operations: vec![operation],
            },
            validate,
            transaction,
            &mut vec![],
        )
    }
}