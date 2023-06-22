use dapi_grpc::platform::v0::Proof;
use drive::query::DriveQuery;

pub struct DapiResponse<T> {
    pub response: T,
    pub proof: Option<Proof>,
}

pub type DocumentQuery<'a> = DriveQuery<'a>;
