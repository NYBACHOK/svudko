use std::sync::{Arc, LazyLock, Weak};

use crux_core::{
    effects::{EffectRouter, Routes, routes::Buffer},
    render::RenderOperation,
};
use tokio::runtime::Runtime;

mod app;
mod handlers;

pub mod resolvers;

pub use app::*;
pub use crux_core::App;

use crate::{handlers::local_dns_sd::LocalDnsHandler, resolvers::dns_sd};

static TOKIO_RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().expect("failed to init runtime"));

#[derive(Clone)]
pub struct EffectRoutes {
    render: Arc<Buffer<RenderOperation>>,
    dns: Arc<LocalDnsHandler>,
}

impl Routes<Application> for EffectRoutes {
    fn new(router: std::sync::Weak<EffectRouter<Application, Self>>) -> Self {
        Self {
            render: Arc::new(Buffer::default()),
            dns: Arc::new(LocalDnsHandler::new(
                Weak::clone(&router),
                dns_sd::Resolver::new().unwrap(), // TODO:
            )),
        }
    }
}

pub struct ApplicationCore {
    router: Arc<EffectRouter<Application, EffectRoutes>>,
}

impl ApplicationCore {
    pub fn new() -> Self {
        let router = EffectRouter::new(crux_core::Core::new(), move |routes: EffectRoutes| {
            move |effect| match effect {
                Effect::Render(request) => routes.render.push(request),
                Effect::DnsSd(request) => routes.dns.process(request),
            }
        });

        Self { router }
    }

    pub fn inner(&self) -> &EffectRouter<Application, EffectRoutes> {
        &*self.router
    }

    pub fn render(&self) -> bool {
        !self.router.routes.render.drain().is_empty()
    }
}
