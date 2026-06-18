use svudko_common::resolver::Operation;

use crate::event::StorageEvent;

#[derive(Clone, Debug)]
pub enum StorageRequest {}

impl Operation for StorageRequest {
    type Output = StorageEvent;
}
