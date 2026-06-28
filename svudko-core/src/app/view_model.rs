use std::collections::HashSet;

use svudko_common::hostname::Hostname;

#[derive(Debug)]
pub struct ViewModel {
    pub discovered_services: Vec<LocalDevices>,
    pub pairing_requests: HashSet<Hostname>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalDevices {
    pub hostname: Hostname,
    pub paired: bool,
}
