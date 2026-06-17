mod shell;

use svudko_resolver_exchange::{ExchangeErrors, event::ExchangeEvent};
use svudko_resolver_sd::{
    DnsSdErrors, event::ServiceDiscoveryEvent, request::ServiceDiscoveryRequest,
};

pub use self::shell::*;

#[derive(Clone, Debug)]
pub enum Event {
    // Shell shared events
    Dns(ServiceDiscoveryRequest),
    Exchange(ExchangeRequest),

    // Core only events
    Core(CoreEvent),
}

#[derive(Clone, Debug)]
pub enum CoreEvent {
    DnsReponses(ServiceDiscoveryEvent),
    Exchange(ExchangeEvent),
    Error(String),
}

impl From<DnsSdErrors> for Event {
    fn from(value: DnsSdErrors) -> Self {
        Self::Core(CoreEvent::Error(value.to_string()))
    }
}

impl From<ExchangeErrors> for Event {
    fn from(value: ExchangeErrors) -> Self {
        Self::Core(CoreEvent::Error(value.to_string()))
    }
}
