use dapi_grpc::platform::v0::GetDocumentsResponse;
use dpp::document::Document;
use drive::drive::verify::RootHash;
use drive::query::DriveQuery;

use crate::error::Error;

pub struct DocumentProof {}

impl DocumentProof {
    pub fn new(request: DriveQuery, response: GetDocumentsResponse) {}
    /// Verifies the given proof and returns the root hash of the GroveDB tree and a vector
    /// of serialized documents if the verification is successful.
    ///
    /// # Arguments
    /// * `proof` - A byte slice representing the proof to be verified.
    ///
    /// # Returns
    /// * On success, returns a tuple containing the root hash of the GroveDB tree and a vector of serialized documents.
    /// * On failure, returns an Error.
    ///
    /// # Errors
    /// This function will return an Error if:
    /// * The start at document is not present in proof and it is expected to be.
    /// * The path query fails to verify against the given proof.
    /// * Converting the element into bytes fails.
    pub fn documents_serialized(&self) -> Result<(RootHash, Vec<Vec<u8>>), Error> {
        todo!()
    }

    /// Verifies a proof for a collection of documents.
    ///
    /// This function takes a slice of bytes `proof` containing a serialized proof,
    /// verifies it, and returns a tuple consisting of the root hash and a vector of deserialized documents.
    ///
    /// # Arguments
    ///
    /// * `proof` - A byte slice representing the proof to be verified.
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    /// * A tuple with the root hash and a vector of deserialized `Document`s, if the proof is valid.
    /// * An `Error` variant, in case the proof verification fails or deserialization error occurs.
    ///
    /// # Errors
    ///
    /// This function will return an `Error` variant if:
    /// 1. The proof verification fails.
    /// 2. There is a deserialization error when parsing the serialized document(s) into `Document` struct(s).
    pub fn documents(&self) -> Result<(RootHash, Vec<Document>), Error> {
        todo!()
    }

    /// Verifies if a document exists at the beginning of a proof,
    /// and returns the root hash and the optionally found document.
    ///
    /// # Arguments
    ///
    /// * `proof` - A byte slice containing the proof data.
    /// * `is_proof_subset` - A boolean indicating whether the proof is a subset query or not.
    /// * `document_id` - A byte_32 array, representing the ID of the document to start at.
    ///
    /// # Returns
    ///
    /// A `Result` with a tuple containing:
    /// * The root hash of the verified proof.
    /// * An `Option<Document>` containing the found document if available.
    ///
    /// # Errors
    ///
    /// This function returns an Error in the following cases:
    /// * If the proof is corrupted (wrong path, wrong key, etc.).
    /// * If the provided proof has an incorrect number of elements.
    pub fn first(
        &self,
        document_id: [u8; 32],
        is_proof_subset: bool,
    ) -> Result<(RootHash, Option<Document>), Error> {
        todo!()
    }
}
