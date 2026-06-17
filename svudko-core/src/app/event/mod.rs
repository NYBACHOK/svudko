mod shell;

use svudko_resolver_exchange::event::ExchangeEvent;
use svudko_resolver_sd::{event::LocalDnsSdEvent, request::LocalDnsSdRequest};

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
