use svudko_common::hostname::Hostname;

use crate::models::TrustedHost;

#[derive(Clone, Debug)]
pub enum StorageEvent {
    Fetch(Vec<TrustedHost>),
    HostAlreadyExists(Hostname),
    HostAdded(TrustedHost),
}
