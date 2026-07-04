use svudko_common::hostname::Hostname;

use crate::models::PairedDevice;

#[derive(Clone, Debug)]
pub enum StorageEvent {
    ClientId(uuid::Uuid),
    Fetch(Vec<PairedDevice>),
    DeviceAlreadyExists(Hostname),
    DeviceAdded(PairedDevice),
}
