use svudko_common::{hostname::Hostname, resolver::Operation};

use crate::event::StorageEvent;

#[derive(Debug, Clone)]
pub enum StorageRequest {
    Fetch,
    NewHost {
        hostname: Hostname,
        identifier: String,
        overwrite: bool,
    },
}

impl Operation for StorageRequest {
    type Output = StorageEvent;
}
