use std::collections::{HashMap, HashSet};

use mdns_sd::{ResolvedService, ScopedIp};

#[derive(Debug, Clone, Default)]
pub struct Model {
    pub load_state: LoadState,
    pub trusted_hosts: HashMap<String, String>,
    pub unknown_signatures : HashMap<String, String>,
    pub dns_sd: DnsSdResult,
}

#[derive(Debug, Clone, Default)]
pub struct DnsSdResult {
    pub enabled_discover: bool,
    pub hostname: Option<String>,
    pub dedicated_search: HashSet<ScopedIp>,
    pub discovered_services: HashMap<String, Box<ResolvedService>>,
}

/// Flags to track resources loaded during first start
#[derive(Debug, Clone, Default)]
pub struct LoadState {
    pub hosts: bool,
}
