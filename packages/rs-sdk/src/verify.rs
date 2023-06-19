use dapi_grpc::platform::v0::GetDocumentsRequest;
pub use drive::drive::verify::{
    ContractVerifier, DataContract, Document, DocumentType, DocumentVerifier, Identity,
    IdentityVerifier, PartialIdentity, QueryVerifier,
};
use drive::{drive::Drive, query::DriveQuery};

#[cfg(feature = "mockall")]
pub use drive::drive::verify::{
    MockContractVerifier, MockDocumentVerifier, MockIdentityVerifier, MockQueryVerifier,
};

struct Verifier();

impl Verifier {
    pub fn for_query<'a>(query: &'a DriveQuery) -> Result<&'a dyn QueryVerifier, crate::Error> {
        Ok(query)
    }
}

impl ContractVerifier for Verifier {
    fn verify_contract(
        proof: &[u8],
        contract_known_keeps_history: Option<bool>,
        is_proof_subset: bool,
        contract_id: [u8; 32],
    ) -> Result<(drive::drive::verify::RootHash, Option<DataContract>), drive::error::Error> {
        Drive::verify_contract(
            proof,
            contract_known_keeps_history,
            is_proof_subset,
            contract_id,
        )
    }
    fn verify_contract_history(
        proof: &[u8],
        contract_id: [u8; 32],
        start_at_date: u64,
        limit: Option<u16>,
        offset: Option<u16>,
    ) -> Result<
        (
            drive::drive::verify::RootHash,
            Option<std::collections::BTreeMap<u64, DataContract>>,
        ),
        drive::error::Error,
    > {
        Drive::verify_contract_history(proof, contract_id, start_at_date, limit, offset)
    }
}

// impl IdentityVerifier for Verifier {}
