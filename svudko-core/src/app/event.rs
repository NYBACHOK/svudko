use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::resolvers::dns_sd::{LocalDnsSdRequest, LocalDnsSdResponse};

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum Event {
    // Shell shared events
    Dns(LocalDnsSdRequest),

    // Core only events
    #[serde(skip)]
    #[facet(skip)]
    DnsReponses(LocalDnsSdResponse),
    #[serde(skip)]
    #[facet(skip)]
    Error(String),
}
