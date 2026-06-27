use svudko_common::{hostname::Hostname, resolver::Operation};

use crate::event::ServiceDiscoveryEvent;

#[derive(Clone, Debug)]
pub enum ServiceDiscoveryRequest {
    EnableService(uuid::Uuid),
    DisableService,
    BeginBrowseForServices,
    StopBrowseForServices,
    FindByHostname(Hostname),
}

impl Operation for ServiceDiscoveryRequest {
    type Output = ServiceDiscoveryEvent;
}
