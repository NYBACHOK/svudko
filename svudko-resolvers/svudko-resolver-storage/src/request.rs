use svudko_common::resolver::Operation;

use crate::{event::StorageEvent, models::TrustedHost};

#[derive(Debug, Clone)]
pub enum StorageRequest {
    Fetch,
    NewHost { host: TrustedHost, overwrite: bool },
}

impl Operation for StorageRequest {
    type Output = StorageEvent;
}
