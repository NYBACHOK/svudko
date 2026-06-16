use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
};

use crate::event::LocalDnsSdEvent;

#[derive(Clone, Debug)]
pub enum LocalDnsSdEffect {
    Enabled,
    Disabled,
    FoundServices(HashMap<String, HashSet<IpAddr>>),
    FoundIps(HashSet<IpAddr>),
}

impl From<LocalDnsSdEvent> for LocalDnsSdEffect {
    fn from(value: LocalDnsSdEvent) -> Self {
        match value {
            LocalDnsSdEvent::Enabled => Self::Enabled,
            LocalDnsSdEvent::Disabled => Self::Disabled,
            LocalDnsSdEvent::FoundServices(hash_map) => Self::FoundServices(
                hash_map
                    .into_iter()
                    .map(|(key, value)| {
                        (
                            key,
                            value
                                .addresses
                                .into_iter()
                                .map(|this| this.to_ip_addr())
                                .collect(),
                        )
                    })
                    .collect(),
            ),
            LocalDnsSdEvent::FoundIps(hash_set) => {
                Self::FoundIps(hash_set.into_iter().map(|this| this.to_ip_addr()).collect())
            }
        }
    }
}
