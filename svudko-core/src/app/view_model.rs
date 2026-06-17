use std::collections::HashSet;

#[derive(Debug)]
pub struct ViewModel {
    pub enabled_discover: bool,
    pub discovered_services: HashSet<String>,
}
