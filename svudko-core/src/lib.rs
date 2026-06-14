use std::sync::{Arc, LazyLock};

use crux_core::{
    effects::{EffectRouter, Routes, routes::Buffer},
    render::RenderOperation,
};
use tokio::runtime::Runtime;

mod app;

pub use app::*;
pub use crux_core::App;

static TOKIO_RUNTIME: LazyLock<Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().expect("failed to init runtime"));

#[derive(Clone)]
pub struct EffectRoutes {
    render: Arc<Buffer<RenderOperation>>,
}

impl Routes<Application> for EffectRoutes {
    fn new(_router: std::sync::Weak<EffectRouter<Application, Self>>) -> Self {
        Self {
            render: Arc::new(Buffer::default()),
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
            }
        });

        Self { router }
    }

    pub fn inner(&self) -> &EffectRouter<Application, EffectRoutes> {
        &*self.router
    }
}
