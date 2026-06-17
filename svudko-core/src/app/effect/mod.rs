use crux_core::{Request, render::RenderOperation};
use svudko_resolver_exchange::request::ExchangeCoreRequest;
use svudko_resolver_sd::request::ServiceDiscoveryRequest;

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
    DnsSd(Request<ServiceDiscoveryRequest>),
    Connection(Request<ExchangeCoreRequest>),
}

impl crux_core::Effect for Effect {}

impl From<Request<RenderOperation>> for Effect {
    fn from(value: Request<RenderOperation>) -> Self {
        Self::Render(value)
    }
}

impl From<Request<ServiceDiscoveryRequest>> for Effect {
    fn from(value: Request<ServiceDiscoveryRequest>) -> Self {
        Self::Core(CoreEffect::DnsSd(value))
    }
}

impl From<Request<ExchangeCoreRequest>> for Effect {
    fn from(value: Request<ExchangeCoreRequest>) -> Self {
        Self::Core(CoreEffect::Connection(value))
    }
}

impl From<Request<CoreErrorEffect>> for Effect {
    fn from(value: Request<CoreErrorEffect>) -> Self {
        Self::Error(value)
    }
}
