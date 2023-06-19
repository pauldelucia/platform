use std::process::ExitCode;

use drive::{contract::Contract, drive::config::DriveConfig, query::DriveQuery};
use rs_sdk::verify::QueryVerifier;

fn main() -> Result<(), ExitCode> {
    let contract = Contract::new();
    let sql_string = "";
    let proof = [0u8; 32];
    let query = DriveQuery::from_sql_expr(sql_string, &contract, &DriveConfig::default())
        .expect("cannot parse query");
    query
        .verify_documents_proof(&proof)
        .expect("document proof verification failed");
    Result::Ok(())
}
