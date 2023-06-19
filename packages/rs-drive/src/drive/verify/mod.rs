use std::collections::BTreeMap;

pub use dpp::{
    data_contract::document_type::DocumentType,
    document::Document,
    identity::{Identity, PartialIdentity},
    prelude::DataContract,
    state_transition::fee::Credits,
};

use crate::error::Error;

/// Contract verification methods on proofs
pub mod contract;
/// Document verification methods on proofs
pub mod document;
/// Identity verification methods on proofs
pub mod identity;
/// Single Document verification methods on proofs
pub mod single_document;

/// Represents the root hash of the grovedb tree
pub type RootHash = [u8; 32];

/// Verify contract
#[cfg_attr(feature = "mockall", mockall::automock)]
pub trait ContractVerifier {
    /// Verifies that the contract is included in the proof.
    ///
    /// # Parameters
    ///
    /// - `proof`: A byte slice representing the proof to be verified.
    /// - `contract_known_keeps_history`: An optional boolean indicating whether the contract keeps a history.
    /// - `is_proof_subset`: A boolean indicating whether to verify a subset of a larger proof.
    /// - `contract_id`: The contract's unique identifier.
    ///
    /// # Returns
    ///
    /// Returns a `Result` with a tuple of `RootHash` and `Option<DataContract>`. The `Option<DataContract>`
    /// represents the verified contract if it exists.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if:
    ///
    /// - The proof is corrupted.
    /// - The GroveDb query fails.
    fn verify_contract(
        proof: &[u8],
        contract_known_keeps_history: Option<bool>,
        is_proof_subset: bool,
        contract_id: [u8; 32],
    ) -> Result<(RootHash, Option<DataContract>), Error>;

    /// Verifies that the contract's history is included in the proof.
    ///
    /// # Parameters
    ///
    /// - `proof`: A byte slice representing the proof to be verified.
    /// - `contract_id`: The contract's unique identifier.
    /// - `start_at_date`: The start date for the contract's history.
    /// - `limit`: An optional limit for the number of items to be retrieved.
    /// - `offset`: An optional offset for the items to be retrieved.
    ///
    /// # Returns
    ///
    /// Returns a `Result` with a tuple of `RootHash` and `Option<BTreeMap<u64, DataContract>>`. The `Option<BTreeMap<u64, DataContract>>`
    /// represents a mapping from dates to contracts if it exists.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if:
    ///
    /// - The proof is corrupted.
    /// - The GroveDb query fails.
    /// - The contract serialization fails.
    fn verify_contract_history(
        proof: &[u8],
        contract_id: [u8; 32],
        start_at_date: u64,
        limit: Option<u16>,
        offset: Option<u16>,
    ) -> Result<(RootHash, Option<BTreeMap<u64, DataContract>>), Error>;
}

/// Verify and parse proofs of some query
///
/// Note that query used for verification must be exactly the same as query used to retrieve the proof.
#[cfg_attr(feature = "mockall", mockall::automock)]
pub trait QueryVerifier {
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
    fn verify_documents_serialized(&self, proof: &[u8]) -> Result<(RootHash, Vec<Vec<u8>>), Error>;

    /// Verifies a proof for a query returning a collection of documents.
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
    fn verify_documents_proof(&self, proof: &[u8]) -> Result<(RootHash, Vec<Document>), Error>;

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
    fn verify_first_document_proof(
        &self,
        proof: &[u8],
        is_proof_subset: bool,
        document_id: [u8; 32],
    ) -> Result<(RootHash, Option<Document>), Error>;
}

/// Verify identity
#[cfg_attr(feature = "mockall", mockall::automock)]
pub trait IdentityVerifier {
    /// Verifies the full identity of a user by their public key hash.
    ///
    /// This function takes a byte slice `proof` and a 20-byte array `public_key_hash` as arguments,
    /// then it verifies the identity of the user with the given public key hash.
    ///
    /// The `proof` should contain the proof of authentication from the user.
    /// The `public_key_hash` should contain the hash of the public key of the user.
    ///
    /// The function first verifies the identity ID associated with the given public key hash
    /// by calling `verify_identity_id_by_public_key_hash()`. It then uses this identity ID to verify
    /// the full identity by calling `verify_full_identity_by_identity_id()`.
    ///
    /// # Returns
    ///
    /// If the verification is successful, it returns a `Result` with a tuple of `RootHash` and
    /// an `Option` of `Identity`. The `RootHash` represents the root hash of GroveDB, and the
    /// `Option<Identity>` represents the full identity of the user if it exists.
    ///
    /// If the verification fails at any point, it will return an `Error`.
    ///
    /// # Errors
    ///
    /// This function will return an `Error` if:
    ///
    /// * The proof of authentication is not valid.
    /// * The public key hash does not correspond to a valid identity ID.
    /// * The identity ID does not correspond to a valid full identity.
    ///
    fn verify_full_identity_by_public_key_hash(
        proof: &[u8],
        public_key_hash: [u8; 20],
    ) -> Result<(RootHash, Option<Identity>), Error>;

    /// Verifies the full identities of multiple users by their public key hashes.
    ///
    /// This function is a generalization of `verify_full_identity_by_public_key_hash`,
    /// which works with a slice of public key hashes instead of a single hash.
    ///
    /// # Parameters
    ///
    /// - `proof`: A byte slice representing the proof of authentication from the users.
    /// - `public_key_hashes`: A reference to a slice of 20-byte arrays, each representing
    ///    a hash of a public key of a user.
    ///
    /// # Generic Parameters
    ///
    /// - `T`: The type of the collection to hold the results, which must be constructible
    ///    from an iterator of tuples of a 20-byte array and an optional `Identity`.
    ///
    /// # Returns
    ///
    /// If the verification is successful, it returns a `Result` with a tuple of `RootHash` and `T`.
    /// The `RootHash` represents the root hash of GroveDB, and `T` represents
    /// the collection of the public key hash and associated identity (if it exists) for each user.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if:
    ///
    /// - The proof of authentication is not valid.
    /// - Any of the public key hashes do not correspond to a valid identity ID.
    /// - Any of the identity IDs do not correspond to a valid full identity.
    ///
    fn verify_full_identities_by_public_key_hashes<
        T: FromIterator<([u8; 20], Option<Identity>)> + 'static,
    >(
        proof: &[u8],
        public_key_hashes: &[[u8; 20]],
    ) -> Result<(RootHash, T), Error>;

    /// Verifies the full identity of a user by their identity ID.
    ///
    /// # Parameters
    ///
    /// - `proof`: A byte slice representing the proof of authentication from the user.
    /// - `is_proof_subset`: A boolean indicating whether the proof is a subset.
    /// - `identity_id`: A 32-byte array representing the identity ID of the user.
    ///
    /// # Returns
    ///
    /// If the verification is successful, it returns a `Result` with a tuple of `RootHash` and
    /// an `Option` of `Identity`. The `RootHash` represents the root hash of GroveDB, and the
    /// `Option<Identity>` represents the full identity of the user if it exists.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if:
    ///
    /// - The proof of authentication is not valid.
    /// - The identity ID does not correspond to a valid full identity.
    /// - The balance, revision, or keys information is missing or incorrect.
    ///
    fn verify_full_identity_by_identity_id(
        proof: &[u8],
        is_proof_subset: bool,
        identity_id: [u8; 32],
    ) -> Result<(RootHash, Option<Identity>), Error>;

    /// Verifies the identity keys of a user by their identity ID.
    ///
    /// # Parameters
    ///
    /// - `proof`: A byte slice representing the proof of authentication from the user.
    /// - `is_proof_subset`: A boolean indicating whether the proof is a subset.
    /// - `identity_id`: A 32-byte array representing the identity ID of the user.
    ///
    /// # Returns
    ///
    /// If the verification is successful, it returns a `Result` with a tuple of `RootHash` and
    /// an `Option` of `PartialIdentity`. The `RootHash` represents the root hash of GroveDB,
    /// and the `Option<PartialIdentity>` represents the partial identity of the user if it exists.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if:
    ///
    /// - The proof of authentication is not valid.
    /// - The identity ID does not correspond to a valid partial identity.
    /// - The keys information is missing or incorrect.
    ///
    fn verify_identity_keys_by_identity_id(
        proof: &[u8],
        is_proof_subset: bool,
        identity_id: [u8; 32],
    ) -> Result<(RootHash, Option<PartialIdentity>), Error>;

    /// Verifies the identity ID of a user by their public key hash.
    ///
    /// # Parameters
    ///
    /// - `proof`: A byte slice representing the proof of authentication from the user.
    /// - `is_proof_subset`: A boolean indicating whether the proof is a subset.
    /// - `public_key_hash`: A 20-byte array representing the hash of the public key of the user.
    ///
    /// # Returns
    ///
    /// If the verification is successful, it returns a `Result` with a tuple of `RootHash` and
    /// an `Option` of a 32-byte array. The `RootHash` represents the root hash of GroveDB,
    /// and the `Option<[u8; 32]>` represents the identity ID of the user if it exists.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if:
    ///
    /// - The proof of authentication is not valid.
    /// - The public key hash does not correspond to a valid identity ID.
    /// - The proved key value is not for the correct path or key in unique key hashes.
    /// - More than one identity ID is found.
    ///
    fn verify_identity_id_by_public_key_hash(
        proof: &[u8],
        is_proof_subset: bool,
        public_key_hash: [u8; 20],
    ) -> Result<(RootHash, Option<[u8; 32]>), Error>;
    /// Verifies the balance of an identity by their identity ID.
    ///
    /// `verify_subset_of_proof` is used to indicate if we want to verify a subset of a bigger proof.
    /// For example, if the proof can prove the balance and the revision, but here we are only interested
    /// in verifying the balance.
    ///
    /// # Parameters
    ///
    /// - `proof`: A byte slice representing the proof of authentication from the user.
    /// - `identity_id`: A 32-byte array representing the identity ID of the user.
    /// - `verify_subset_of_proof`: A boolean indicating whether we are verifying a subset of a larger proof.
    ///
    /// # Returns
    ///
    /// If the verification is successful, it returns a `Result` with a tuple of `RootHash` and
    /// an `Option<u64>`. The `RootHash` represents the root hash of GroveDB, and the
    /// `Option<u64>` represents the balance of the user's identity if it exists.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if:
    ///
    /// - The proof of authentication is not valid.
    /// - The identity ID does not correspond to a valid balance.
    /// - The proved key value is not for the correct path or key in balances.
    /// - More than one balance is found.
    ///
    fn verify_identity_balance_for_identity_id(
        proof: &[u8],
        identity_id: [u8; 32],
        verify_subset_of_proof: bool,
    ) -> Result<(RootHash, Option<u64>), Error>;

    /// Verifies the balances of multiple identities by their identity IDs.
    ///
    /// `is_proof_subset` is used to indicate if we want to verify a subset of a bigger proof.
    /// For example, if the proof can prove the balances and revisions, but here we are only
    /// interested in verifying the balances.
    ///
    /// # Parameters
    ///
    /// - `proof`: A byte slice representing the proofs of authentication from the users.
    /// - `is_proof_subset`: A boolean indicating whether we are verifying a subset of a larger proof.
    /// - `identity_ids`: A slice of 32-byte arrays representing the identity IDs of the users.
    ///
    /// # Returns
    ///
    /// If the verification is successful, it returns a `Result` with a tuple of `RootHash` and
    /// a generic collection `T` of tuples. Each tuple in `T` consists of a 32-byte array
    /// representing an identity ID and an `Option<Credits>`. The `Option<Credits>` represents
    /// the balance of the respective identity if it exists.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if:
    ///
    /// - The proof of authentication is not valid.
    /// - Any of the identity IDs does not correspond to a valid balance.
    /// - The number of proved key values does not match the number of identity IDs provided.
    /// - The value size of the balance is incorrect.
    ///
    fn verify_identity_balances_for_identity_ids<
        T: FromIterator<([u8; 32], Option<Credits>)> + 'static,
    >(
        proof: &[u8],
        is_proof_subset: bool,
        identity_ids: &[[u8; 32]],
    ) -> Result<(RootHash, T), Error>;

    /// Verifies the identity IDs of multiple identities by their public key hashes.
    ///
    /// `is_proof_subset` is used to indicate if we want to verify a subset of a bigger proof.
    /// For example, if the proof can prove the identity IDs and revisions, but here we are only
    /// interested in verifying the identity IDs.
    ///
    /// # Parameters
    ///
    /// - `proof`: A byte slice representing the proofs of authentication from the users.
    /// - `is_proof_subset`: A boolean indicating whether we are verifying a subset of a larger proof.
    /// - `public_key_hashes`: A slice of 20-byte arrays representing the public key hashes of the users.
    ///
    /// # Returns
    ///
    /// If the verification is successful, it returns a `Result` with a tuple of `RootHash` and
    /// a generic collection `T` of tuples. Each tuple in `T` consists of a 20-byte array
    /// representing a public key hash and an `Option<[u8; 32]>`. The `Option<[u8; 32]>` represents
    /// the identity ID of the respective identity if it exists.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if:
    ///
    /// - The proof of authentication is not valid.
    /// - Any of the public key hashes does not correspond to a valid identity ID.
    /// - The number of proved key values does not match the number of public key hashes provided.
    /// - The value size of the identity ID is incorrect.
    ///
    fn verify_identity_ids_by_public_key_hashes<
        T: FromIterator<([u8; 20], Option<[u8; 32]>)> + 'static,
    >(
        proof: &[u8],
        is_proof_subset: bool,
        public_key_hashes: &[[u8; 20]],
    ) -> Result<(RootHash, T), Error>;
}

/// Verify single document
#[cfg_attr(feature = "mockall", mockall::automock)]
pub trait DocumentVerifier {
    /// Verifies the proof of a document while keeping it serialized.
    ///
    /// `is_subset` indicates if the function should verify a subset of a larger proof.
    ///
    /// # Parameters
    ///
    /// - `is_subset`: A boolean indicating whether to verify a subset of a larger proof.
    /// - `proof`: A byte slice representing the proof to be verified.
    ///
    /// # Returns
    ///
    /// Returns a `Result` with a tuple of `RootHash` and `Option<Vec<u8>>`. The `Option<Vec<u8>>`
    /// represents the serialized document if it exists.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if:
    ///
    /// - The proof is corrupted.
    /// - The GroveDb verification fails.
    /// - The elements returned are not items, the proof is incorrect.
    fn verify_proof_keep_serialized(
        &self,
        is_subset: bool,
        proof: &[u8],
    ) -> Result<(RootHash, Option<Vec<u8>>), Error>;
    /// Verifies the proof of a single document query.
    ///
    /// `is_subset` indicates if the function should verify a subset of a larger proof.
    ///
    /// # Parameters
    ///
    /// - `is_subset`: A boolean indicating whether to verify a subset of a larger proof.
    /// - `proof`: A byte slice representing the proof to be verified.
    /// - `document_type`: The type of the document being verified.
    ///
    /// # Returns
    ///
    /// Returns a `Result` with a tuple of `RootHash` and `Option<Document>`. The `Option<Document>`
    /// represents the deserialized document if it exists.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if:
    ///
    /// - The proof is corrupted.
    /// - The GroveDb query fails.
    /// - The document serialization fails.
    fn verify_proof(
        &self,
        is_subset: bool,
        proof: &[u8],
        document_type: &DocumentType,
    ) -> Result<(RootHash, Option<Document>), Error>;
}
