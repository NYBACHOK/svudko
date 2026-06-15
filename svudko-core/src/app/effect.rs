use crux_core::{Request, render::RenderOperation};

use crate::resolvers::dns_sd::LocalDnsSdRequest;

#[derive(Debug)]
pub enum Effect {
    Render(Request<RenderOperation>),
    DnsSd(Request<LocalDnsSdRequest>),
}

impl crux_core::Effect for Effect {}

impl From<Request<RenderOperation>> for Effect {
    fn from(value: Request<RenderOperation>) -> Self {
        Self::Render(value)
    }
}

impl From<Request<LocalDnsSdRequest>> for Effect {
    fn from(value: Request<LocalDnsSdRequest>) -> Self {
        Self::DnsSd(value)
    }
}
