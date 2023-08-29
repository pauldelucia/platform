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

//! Grove Operations.
//!
//! Defines and implements in Drive functions pertinent to groveDB operations.
//!

/// Grove insert operation
pub mod grove_insert;

/// Grove insert operation into an empty tree
pub mod grove_insert_empty_tree;

/// Grove insert operation into an empty sum tree
pub mod grove_insert_empty_sum_tree;

/// Grove insert operation, but only if it doesn't already exist
pub mod grove_insert_if_not_exists;

/// Grove delete operation
pub mod grove_delete;

/// Fetch raw grove data
pub mod grove_get_raw;

/// Fetch raw grove data if it exists
pub mod grove_get_raw_optional;

/// Fetch u64 value from encoded variable vector in raw grove data
pub mod grove_get_raw_value_u64_from_encoded_var_vec;

/// Grove get operation
pub mod grove_get;

/// Serialized results from grove path query
pub mod grove_get_path_query_serialized_results;

/// Grove path query operation
pub mod grove_get_path_query;

/// Grove path query operation with optional return value
pub mod grove_get_path_query_with_optional;

/// Fetch raw data from grove path query with optional return value
pub mod grove_get_raw_path_query_with_optional;

/// Fetch raw data from grove path query
pub mod grove_get_raw_path_query;

/// Proved path query in grove
pub mod grove_get_proved_path_query;

/// Get total value from sum tree in grove
pub mod grove_get_sum_tree_total_value;

/// Check if raw data exists in grove
pub mod grove_has_raw;

/// Batch insert operation into empty tree
pub mod batch_insert_empty_tree;

/// Batch insert operation into empty tree, but only if it doesn't already exist
pub mod batch_insert_empty_tree_if_not_exists;

/// Batch insert operation into empty tree, but only if it doesn't exist and check existing operations
pub mod batch_insert_empty_tree_if_not_exists_check_existing_operations;

/// Batch insert operation
pub mod batch_insert;

/// Batch insert operation, but only if it doesn't already exist
pub mod batch_insert_if_not_exists;

/// Batch insert operation, but only if the value has changed
pub mod batch_insert_if_changed_value;

/// Batch delete operation
pub mod batch_delete;

/// Batch remove raw data operation
pub mod batch_remove_raw;

/// Batch delete operation up the tree while it's empty
pub mod batch_delete_up_tree_while_empty;

/// Batch refresh reference operation
pub mod batch_refresh_reference;

/// Apply grove operation
pub mod grove_apply_operation;

/// Apply batch grove operation
pub mod grove_apply_batch;

/// Apply batch grove operation with additional costs
pub mod grove_apply_batch_with_add_costs;

/// Apply partial batch grove operation
pub mod grove_apply_partial_batch;

/// Apply partial batch grove operation with additional costs
pub mod grove_apply_partial_batch_with_add_costs;

/// Get cost of grove batch operations
pub mod grove_batch_operations_costs;

use costs::CostContext;

use grovedb::EstimatedLayerInformation;

use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;
use crate::fee::op::LowLevelDriveOperation::CalculatedCostOperation;

use grovedb::Error as GroveError;

use intmap::IntMap;

/// Pushes an operation's `OperationCost` to `drive_operations` given its `CostContext`
/// and returns the operation's return value.
fn push_drive_operation_result<T>(
    cost_context: CostContext<Result<T, GroveError>>,
    drive_operations: &mut Vec<LowLevelDriveOperation>,
) -> Result<T, Error> {
    let CostContext { value, cost } = cost_context;
    drive_operations.push(CalculatedCostOperation(cost));
    value.map_err(Error::GroveDB)
}

/// Pushes an operation's `OperationCost` to `drive_operations` given its `CostContext`
/// if `drive_operations` is given. Returns the operation's return value.
fn push_drive_operation_result_optional<T>(
    cost_context: CostContext<Result<T, GroveError>>,
    drive_operations: Option<&mut Vec<LowLevelDriveOperation>>,
) -> Result<T, Error> {
    let CostContext { value, cost } = cost_context;
    if let Some(drive_operations) = drive_operations {
        drive_operations.push(CalculatedCostOperation(cost));
    }
    value.map_err(Error::GroveDB)
}
/// is subtree?
pub type IsSubTree = bool;
/// is sum subtree?
pub type IsSumSubTree = bool;
/// is sum tree?
pub type IsSumTree = bool;

/// batch delete apply type
pub enum BatchDeleteApplyType {
    /// stateless batch delete
    StatelessBatchDelete {
        is_sum_tree: bool,
        estimated_value_size: u32,
    },
    /// stateful batch delete
    StatefulBatchDelete {
        is_known_to_be_subtree_with_sum: Option<(IsSubTree, IsSumSubTree)>,
    },
}

/// batch delete up tree apply type
pub enum BatchDeleteUpTreeApplyType {
    /// stateless batch delete
    StatelessBatchDelete {
        estimated_layer_info: IntMap<EstimatedLayerInformation>,
    },
    /// stateful batch delete
    StatefulBatchDelete {
        is_known_to_be_subtree_with_sum: Option<(IsSubTree, IsSumSubTree)>,
    },
}

/// batch insert tree apply type
#[derive(Clone, Copy)]
pub enum BatchInsertTreeApplyType {
    /// stateless batch insert tree
    StatelessBatchInsertTree {
        in_tree_using_sums: bool,
        is_sum_tree: bool,
        flags_len: FlagsLen,
    },
    /// stateful batch insert tree
    StatefulBatchInsertTree,
}

impl BatchInsertTreeApplyType {
    /// to direct query type
    pub(crate) fn to_direct_query_type(&self) -> DirectQueryType {
        match self {
            BatchInsertTreeApplyType::StatelessBatchInsertTree {
                in_tree_using_sums,
                is_sum_tree,
                flags_len,
            } => DirectQueryType::StatelessDirectQuery {
                in_tree_using_sums: *in_tree_using_sums,
                query_target: QueryTarget::QueryTargetTree(*flags_len, *is_sum_tree),
            },
            BatchInsertTreeApplyType::StatefulBatchInsertTree => {
                DirectQueryType::StatefulDirectQuery
            }
        }
    }
}

/// batch insert apply type
pub enum BatchInsertApplyType {
    /// stateless
    StatelessBatchInsert {
        in_tree_using_sums: bool,
        target: QueryTarget,
    },
    /// stateful
    StatefulBatchInsert,
}

impl BatchInsertApplyType {
    /// to direct query type
    pub(crate) fn to_direct_query_type(&self) -> DirectQueryType {
        match self {
            BatchInsertApplyType::StatelessBatchInsert {
                in_tree_using_sums,
                target,
            } => DirectQueryType::StatelessDirectQuery {
                in_tree_using_sums: *in_tree_using_sums,
                query_target: *target,
            },
            BatchInsertApplyType::StatefulBatchInsert => DirectQueryType::StatefulDirectQuery,
        }
    }
}

/// flags length
pub type FlagsLen = u32;

/// query target
#[derive(Clone, Copy)]
pub enum QueryTarget {
    /// tree
    QueryTargetTree(FlagsLen, IsSumTree),
    /// value
    QueryTargetValue(u32),
}

impl QueryTarget {
    /// get query target length
    pub(crate) fn len(&self) -> u32 {
        match self {
            QueryTarget::QueryTargetTree(flags_len, is_sum_tree) => {
                let len = if *is_sum_tree { 11 } else { 3 };
                *flags_len + len
            }
            QueryTarget::QueryTargetValue(len) => *len,
        }
    }
}

/// direct query type
#[derive(Clone, Copy)]
pub enum DirectQueryType {
    /// stateless
    StatelessDirectQuery {
        in_tree_using_sums: bool,
        query_target: QueryTarget,
    },
    /// stateful
    StatefulDirectQuery,
}

impl From<DirectQueryType> for QueryType {
    fn from(value: DirectQueryType) -> Self {
        match value {
            DirectQueryType::StatelessDirectQuery {
                in_tree_using_sums,
                query_target,
            } => QueryType::StatelessQuery {
                in_tree_using_sums,
                query_target,
                estimated_reference_sizes: vec![],
            },
            DirectQueryType::StatefulDirectQuery => QueryType::StatefulQuery,
        }
    }
}

impl DirectQueryType {
    /// add reference sizes to direct query type
    #[allow(dead_code)]
    pub(crate) fn add_reference_sizes(self, reference_sizes: Vec<u32>) -> QueryType {
        match self {
            DirectQueryType::StatelessDirectQuery {
                in_tree_using_sums,
                query_target,
            } => QueryType::StatelessQuery {
                in_tree_using_sums,
                query_target,
                estimated_reference_sizes: reference_sizes,
            },
            DirectQueryType::StatefulDirectQuery => QueryType::StatefulQuery,
        }
    }
}

/// query type (sam is downgraded from A+ manager to A- for making me do all these docs)
#[derive(Clone)]
pub enum QueryType {
    /// stateless
    StatelessQuery {
        in_tree_using_sums: bool,
        query_target: QueryTarget,
        estimated_reference_sizes: Vec<u32>,
    },
    /// stateful
    StatefulQuery,
}

impl From<BatchDeleteApplyType> for QueryType {
    fn from(value: BatchDeleteApplyType) -> Self {
        match value {
            BatchDeleteApplyType::StatelessBatchDelete {
                is_sum_tree,
                estimated_value_size,
            } => QueryType::StatelessQuery {
                in_tree_using_sums: is_sum_tree,
                query_target: QueryTarget::QueryTargetValue(estimated_value_size),
                estimated_reference_sizes: vec![],
            },
            BatchDeleteApplyType::StatefulBatchDelete { .. } => QueryType::StatefulQuery,
        }
    }
}

impl From<&BatchDeleteApplyType> for QueryType {
    fn from(value: &BatchDeleteApplyType) -> Self {
        match value {
            BatchDeleteApplyType::StatelessBatchDelete {
                is_sum_tree,
                estimated_value_size,
            } => QueryType::StatelessQuery {
                in_tree_using_sums: *is_sum_tree,
                query_target: QueryTarget::QueryTargetValue(*estimated_value_size),
                estimated_reference_sizes: vec![],
            },
            BatchDeleteApplyType::StatefulBatchDelete { .. } => QueryType::StatefulQuery,
        }
    }
}

impl From<BatchDeleteApplyType> for DirectQueryType {
    fn from(value: BatchDeleteApplyType) -> Self {
        match value {
            BatchDeleteApplyType::StatelessBatchDelete {
                is_sum_tree,
                estimated_value_size,
            } => DirectQueryType::StatelessDirectQuery {
                in_tree_using_sums: is_sum_tree,
                query_target: QueryTarget::QueryTargetValue(estimated_value_size),
            },
            BatchDeleteApplyType::StatefulBatchDelete { .. } => {
                DirectQueryType::StatefulDirectQuery
            }
        }
    }
}

impl From<&BatchDeleteApplyType> for DirectQueryType {
    fn from(value: &BatchDeleteApplyType) -> Self {
        match value {
            BatchDeleteApplyType::StatelessBatchDelete {
                is_sum_tree,
                estimated_value_size,
            } => DirectQueryType::StatelessDirectQuery {
                in_tree_using_sums: *is_sum_tree,
                query_target: QueryTarget::QueryTargetValue(*estimated_value_size),
            },
            BatchDeleteApplyType::StatefulBatchDelete { .. } => {
                DirectQueryType::StatefulDirectQuery
            }
        }
    }
}
