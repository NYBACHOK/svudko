use std::sync::{Arc, LazyLock, Weak};

use crux_core::{
    effects::{EffectRouter, Routes},
    render::RenderOperation,
};
use tokio::runtime::Runtime;

mod app;
mod handlers;

pub mod resolvers;

pub use app::*;
pub use crux_core::App;

use crate::app::effect::CoreEffect;

use crate::app::effect::Effect as RustCoreEffect;

static TOKIO_RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().expect("failed to init runtime"));

pub trait CruxShell {
    fn process_effects(&self, effect: Effect);
}

#[derive(Debug)]
pub enum Effect {
    Render(RenderOperation),
    Error(String),
}

impl From<RustCoreEffect> for Effect {
    fn from(value: RustCoreEffect) -> Self {
        match value {
            RustCoreEffect::Render(request) => Self::Render(request.operation),
            RustCoreEffect::Error(e) => Self::Error(e.operation.0),
            RustCoreEffect::Core(_) => unreachable!("effect handled by core"),
        }
    }
}

#[derive(Clone)]
pub struct EffectRoutes {
    dns: Arc<handlers::dns_sd::Handler>,
    connection: Arc<handlers::exchange::Handler>,
}

impl Routes<Application> for EffectRoutes {
    fn new(router: std::sync::Weak<EffectRouter<Application, Self>>) -> Self {
        Self {
            dns: Arc::new(handlers::dns_sd::Handler::new(
                Weak::clone(&router),
                resolvers::dns_sd::Resolver::new().unwrap(), // TODO:
            )),
            connection: Arc::new(handlers::exchange::Handler::new(
                Weak::clone(&router),
                resolvers::exchange::Resolver::new().unwrap(),
            )),
        }
    }
}

pub struct ApplicationCore {
    router: Arc<EffectRouter<Application, EffectRoutes>>,
}

impl ApplicationCore {
    pub fn new<T: CruxShell + Send + Sync + 'static>(shell: Arc<T>) -> Self {
        let router = EffectRouter::new(crux_core::Core::new(), move |routes: EffectRoutes| {
            move |effect| match effect {
                RustCoreEffect::Core(CoreEffect::DnsSd(request)) => routes.dns.process(request),
                RustCoreEffect::Core(CoreEffect::Connection(request)) => {
                    routes.connection.process(request)
                }
                effect => shell.process_effects(effect.into()),
            }
        });

        Self { router }
    }

    pub fn inner(&self) -> &EffectRouter<Application, EffectRoutes> {
        &*self.router
    }
}
