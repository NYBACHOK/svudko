use std::collections::{HashMap, HashSet};

use mdns_sd::ResolvedService;
use svudko_common::hostname::Hostname;

#[derive(Debug, Clone, Default)]
pub struct Model {
    pub load_state: LoadState,
    pub trusted_signatures: HashSet<String>,
    pub unknown_signatures: HashMap<Hostname, String>,
    pub dns_sd: DnsSdResult,
}

#[derive(Debug, Clone, Default)]
pub struct DnsSdResult {
    pub enabled_discover: bool,
    pub discovered_services: HashMap<String, Box<ResolvedService>>,
}

/// Flags to track resources loaded during first start
#[derive(Debug, Clone, Default)]
pub struct LoadState {
    pub hosts: bool,
}
