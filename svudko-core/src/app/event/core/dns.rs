use std::collections::{HashMap, HashSet};

use mdns_sd::{ResolvedService, ScopedIp};

#[derive(Clone, Debug)]
pub enum LocalDnsSdEvent {
    Enabled,
    Disabled,
    FoundServices(HashMap<String, Box<ResolvedService>>),
    FoundIps(HashSet<ScopedIp>),
}
