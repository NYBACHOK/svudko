use std::collections::{HashMap, HashSet};

use mdns_sd::{ResolvedService, ScopedIp};

#[derive(Debug, Clone, Default)]
pub struct Model {
    pub dns_sd: DnsSdResult,
    pub connected : HashSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DnsSdResult {
    pub enabled_discover: bool,
    pub hostname: Option<String>,
    pub dedicated_search: HashSet<ScopedIp>,
    pub discovered_services: HashMap<String, Box<ResolvedService>>,
}
