use crux_core::{Request, render::RenderOperation};
use svudko_resolver_exchange::request::ExchangeCoreRequest;
use svudko_resolver_sd::request::LocalDnsSdRequest;

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
    DnsSd(Request<LocalDnsSdRequest>),
    Connection(Request<ExchangeCoreRequest>),
}

impl crux_core::Effect for Effect {}

impl From<Request<RenderOperation>> for Effect {
    fn from(value: Request<RenderOperation>) -> Self {
        Self::Render(value)
    }
}

impl From<Request<LocalDnsSdRequest>> for Effect {
    fn from(value: Request<LocalDnsSdRequest>) -> Self {
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
