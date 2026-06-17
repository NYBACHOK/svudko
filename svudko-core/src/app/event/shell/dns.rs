use crux_core::capability::Operation;

use crate::{event::LocalDnsSdEvent, resolvers::dns_sd::DnsSdErrors};

#[derive(Clone, Debug)]
pub enum LocalDnsSdRequest {
    EnableService,
    DisableService,
    BrowseForServices,
    FindByHostname(String),
}

impl Operation for LocalDnsSdRequest {
    type Output = Result<LocalDnsSdEvent, DnsSdErrors>;
}
