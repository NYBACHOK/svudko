use svudko_resolver_exchange::{errors::ExchangeErrors, event::ExchangeEvent};
use svudko_resolver_sd::{ServiceDiscoveryErrors, event::ServiceDiscoveryEvent};
use svudko_resolver_storage::{StorageErrors, event::StorageEvent, request::StorageRequest};

use crate::event::exchange::ExchangeRequestEvent;

pub mod exchange;

#[derive(Debug)]
pub enum Event {
    Initialize,
    // Shell shared events
    Exchange(ExchangeRequestEvent),
    Storage(StorageRequest),

    // Core only events
    Core(CoreEvent),
}

#[derive(Debug)]
pub enum CoreEvent {
    ServiceDiscovery(ServiceDiscoveryEvent),
    Exchange(ExchangeEvent),
    Storage(StorageEvent),
    Error(String),
}

impl From<ServiceDiscoveryErrors> for Event {
    fn from(value: ServiceDiscoveryErrors) -> Self {
        Self::Core(CoreEvent::Error(value.to_string()))
    }
}

impl From<ExchangeErrors> for Event {
    fn from(value: ExchangeErrors) -> Self {
        Self::Core(CoreEvent::Error(value.to_string()))
    }
}

impl From<StorageErrors> for Event {
    fn from(value: StorageErrors) -> Self {
        Self::Core(CoreEvent::Error(value.to_string()))
    }
}

impl From<StorageRequest> for Event {
    fn from(value: StorageRequest) -> Self {
        Self::Storage(value)
    }
}

impl From<StorageEvent> for Event {
    fn from(value: StorageEvent) -> Self {
        Self::Core(CoreEvent::Storage(value))
    }
}

impl From<ExchangeRequestEvent> for Event {
    fn from(value: ExchangeRequestEvent) -> Self {
        Self::Exchange(value)
    }
}

impl From<ExchangeEvent> for Event {
    fn from(value: ExchangeEvent) -> Self {
        Self::Core(CoreEvent::Exchange(value))
    }
}

impl From<ServiceDiscoveryEvent> for Event {
    fn from(value: ServiceDiscoveryEvent) -> Self {
        Self::Core(CoreEvent::ServiceDiscovery(value))
    }
}
