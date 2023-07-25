use crate::state_transition::public_key_in_creation::fields::*;
use crate::state_transition::public_key_in_creation::v0::IdentityPublicKeyInCreationV0;
use crate::state_transition::StateTransitionFieldTypes;

impl StateTransitionFieldTypes for IdentityPublicKeyInCreationV0 {
    fn signature_property_paths() -> Vec<&'static str> {
        vec![SIGNATURE]
    }

    fn identifiers_property_paths() -> Vec<&'static str> {
        vec![]
    }

    fn binary_property_paths() -> Vec<&'static str> {
        BINARY_DATA_FIELDS.to_vec()
    }
}
