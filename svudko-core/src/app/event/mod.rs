mod shell;

use svudko_resolver_exchange::{ExchangeErrors, event::ExchangeEvent};
use svudko_resolver_sd::{DnsSdErrors, event::LocalDnsSdEvent, request::LocalDnsSdRequest};

pub use self::shell::*;

#[derive(Clone, Debug)]
pub enum Event {
    // Shell shared events
    Dns(LocalDnsSdRequest),
    Exchange(ExchangeRequest),

    // Core only events
    Core(CoreEvent),
}

#[derive(Clone, Debug)]
pub enum CoreEvent {
    DnsReponses(LocalDnsSdEvent),
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
