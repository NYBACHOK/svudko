use crate::resolvers::dns_sd::{LocalDnsSdRequest, LocalDnsSdResponse};

#[derive(Clone, Debug)]
#[repr(C)]
pub enum Event {
    // Shell shared events
    Dns(LocalDnsSdRequest),

    // Core only events
    DnsReponses(LocalDnsSdResponse),
    Error(String),
}
