pub use data_contract::*;
pub use data_contract_factory::*;
pub use generate_data_contract::*;

mod data_contract;
pub mod errors;
pub mod extra;

mod data_contract_facade;

pub mod contract_config;
mod data_contract_factory;
pub mod document_type;
pub mod enrich_with_base_schema;
mod generate_data_contract;
pub mod get_binary_properties_from_schema;
pub mod get_property_definition_by_path;
pub mod serialization;
pub mod state_transition;
pub mod structure_validation;
pub mod validation;

pub mod property_names {
    pub const PROTOCOL_VERSION: &str = "protocolVersion";
    pub const ID: &str = "$id";
    pub const OWNER_ID: &str = "ownerId";
    pub const VERSION: &str = "version";
    pub const SCHEMA: &str = "$schema";
    pub const DOCUMENTS: &str = "documents";
    pub const DEFINITIONS: &str = "$defs";
    pub const ENTROPY: &str = "entropy"; // not a data contract field actually but at some point it can be there for some time
}

pub use data_contract_facade::DataContractFacade;
