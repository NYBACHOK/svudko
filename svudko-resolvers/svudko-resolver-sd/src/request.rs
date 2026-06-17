use crux_core::capability::Operation;

use crate::event::ServiceDiscoveryEvent;

#[derive(Clone, Debug)]
pub enum ServiceDiscoveryRequest {
    EnableService,
    DisableService,
    BrowseForServices,
    FindByHostname(String),
}

impl Operation for ServiceDiscoveryRequest {
    type Output = ServiceDiscoveryEvent;
}
