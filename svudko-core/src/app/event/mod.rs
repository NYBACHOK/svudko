use svudko_resolver_exchange::{
    errors::ExchangeErrors, event::ExchangeEvent, request::ExchangeRequest,
};
use svudko_resolver_sd::{
    DnsSdErrors, event::ServiceDiscoveryEvent, request::ServiceDiscoveryRequest,
};
use svudko_resolver_storage::{StorageErrors, event::StorageEvent, request::StorageRequest};

#[derive(Clone, Debug)]
pub enum Event {
    Initialize,
    // Shell shared events
    ServiceDiscovery(ServiceDiscoveryRequest),
    Exchange(ExchangeRequest),
    Storage(StorageRequest),

    // Core only events
    Core(CoreEvent),
}

#[derive(Clone, Debug)]
pub enum CoreEvent {
    DnsReponses(ServiceDiscoveryEvent),
    Exchange(ExchangeEvent),
    Storage(StorageEvent),
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

impl From<ExchangeRequest> for Event {
    fn from(value: ExchangeRequest) -> Self {
        Self::Exchange(value)
    }
}

impl From<ExchangeEvent> for Event {
    fn from(value: ExchangeEvent) -> Self {
        Self::Core(CoreEvent::Exchange(value))
    }
}

impl From<ServiceDiscoveryRequest> for Event {
    fn from(value: ServiceDiscoveryRequest) -> Self {
        Self::ServiceDiscovery(value)
    }
}

impl From<ServiceDiscoveryEvent> for Event {
    fn from(value: ServiceDiscoveryEvent) -> Self {
        Self::Core(CoreEvent::DnsReponses(value))
    }
}
