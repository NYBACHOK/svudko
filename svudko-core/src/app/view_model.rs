use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct ViewModel {
    pub discovered_services: HashSet<String>,
    pub unknown_signatures: HashMap<String, String>,
}
