use crate::models::TrustedHost;

#[derive(Clone, Debug)]
pub enum StorageEvent {
    Fetch(Vec<TrustedHost>),
    HostAlreadyExists(String),
    HostAdded(TrustedHost),
}
