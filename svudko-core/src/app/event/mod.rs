mod core;
mod shell;

pub use self::{core::*, shell::*};

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
