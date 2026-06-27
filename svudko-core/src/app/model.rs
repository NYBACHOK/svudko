use std::collections::{HashMap, HashSet};

use svudko_common::hostname::Hostname;
use svudko_resolver_sd::models::LocalService;

#[derive(Debug, Clone, Default)]
pub struct Model {
    pub session_id: SessionId,
    pub load_state: LoadState,
    pub trusted_signatures: HashSet<String>,
    pub unknown_signatures: HashMap<Hostname, String>,
    pub discovered_services: HashMap<Hostname, LocalService>,
}

/// Flags to track resources loaded during first start
#[derive(Debug, Clone, Default)]
pub struct LoadState {
    pub hosts: bool,
}

#[derive(Debug, Clone)]
pub struct SessionId {
    id: uuid::Uuid,
    base64: String,
}

impl SessionId {
    pub fn new() -> Self {
        let id = uuid::Uuid::new_v4();

        let base64 = data_encoding::BASE64.encode(id.as_bytes());

        Self { id, base64 }
    }

    pub fn uuid(&self) -> uuid::Uuid {
        self.id
    }

    pub fn base64_repr(&self) -> &str {
        &self.base64
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}
