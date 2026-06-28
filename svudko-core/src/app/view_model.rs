use std::collections::HashMap;

use svudko_common::hostname::Hostname;

#[derive(Debug)]
pub struct ViewModel {
    pub discovered_services: Vec<LocalDevices>,
    pub unknown_signatures: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalDevices {
    pub hostname: Hostname,
    pub paired: bool,
}
