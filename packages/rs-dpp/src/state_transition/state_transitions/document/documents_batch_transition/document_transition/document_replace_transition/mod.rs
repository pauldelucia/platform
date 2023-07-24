mod v0;

use crate::state_transition::documents_batch_transition::document_base_transition::DocumentBaseTransition;
use crate::state_transition::documents_batch_transition::document_transition::DocumentTransitionMethodsV0;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
pub use v0::*;

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, PartialEq)]
pub enum DocumentReplaceTransition {
    V0(DocumentReplaceTransitionV0),
}

impl DocumentTransitionMethodsV0 for DocumentReplaceTransition {
    fn base(&self) -> &DocumentBaseTransition {
        match self {
            DocumentReplaceTransition::V0(v0) => &v0.base,
        }
    }
}
