mod core;
mod shell;

pub use self::{core::*, shell::*};

#[derive(Clone, Debug)]
pub enum Event {
    // Shell shared events
    Dns(LocalDnsSdRequest),

    // Core only events
    Core(CoreEvent),
}

#[derive(Clone, Debug)]
pub enum CoreEvent {
    DnsReponses(LocalDnsSdEvent),
    QuickConnection(ExchangeEvent),
    Error(String),
}
