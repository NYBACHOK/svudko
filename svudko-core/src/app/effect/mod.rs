use crux_core::{Request, render::RenderOperation};
use svudko_resolver_exchange::request::ExchangeRequest;
use svudko_resolver_sd::request::ServiceDiscoveryRequest;
use svudko_resolver_storage::request::StorageRequest;

mod error;

pub use self::error::*;

#[derive(Debug)]
pub enum Effect {
    Render(Request<RenderOperation>),
    Error(Request<CoreErrorEffect>),
    Core(CoreEffect),
}

#[derive(Debug)]
pub enum CoreEffect {
    ServiceDiscovery(Request<ServiceDiscoveryRequest>),
    Connection(Request<ExchangeRequest>),
    Storage(Request<StorageRequest>),
}

impl crux_core::Effect for Effect {}

impl From<Request<RenderOperation>> for Effect {
    fn from(value: Request<RenderOperation>) -> Self {
        Self::Render(value)
    }
}

impl From<Request<ServiceDiscoveryRequest>> for Effect {
    fn from(value: Request<ServiceDiscoveryRequest>) -> Self {
        Self::Core(CoreEffect::ServiceDiscovery(value))
    }
}

impl From<Request<ExchangeRequest>> for Effect {
    fn from(value: Request<ExchangeRequest>) -> Self {
        Self::Core(CoreEffect::Connection(value))
    }
}

impl From<Request<StorageRequest>> for Effect {
    fn from(value: Request<StorageRequest>) -> Self {
        Self::Core(CoreEffect::Storage(value))
    }
}

impl From<Request<CoreErrorEffect>> for Effect {
    fn from(value: Request<CoreErrorEffect>) -> Self {
        Self::Error(value)
    }
}
