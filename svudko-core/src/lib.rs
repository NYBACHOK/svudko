use std::sync::{Arc, Weak};

use crux_core::{
    effects::{EffectRouter, Routes},
    render::RenderOperation,
};
use svudko_common::resolver::HandlerResolver;
use svudko_resolver_exchange::{
    ExchangeResolver, ExchangeResolverOptions, event::ExchangeEvent, request::ExchangeRequest,
};
use svudko_resolver_sd::{SdResolver, SdResolverOptions, request::ServiceDiscoveryRequest};

mod app;
mod handler;

pub use app::*;
pub use crux_core::App;
use svudko_resolver_storage::{StorageResolver, request::StorageRequest};

use crate::{
    app::{effect::CoreEffect, event::Event, view_model::ViewModel},
    handler::Handler,
};

use crate::app::effect::Effect as RustCoreEffect;

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
    dns: Arc<Handler<ServiceDiscoveryRequest>>,
    connection: Arc<Handler<ExchangeRequest>>,
    storage: Arc<Handler<StorageRequest>>,
}

impl Routes<Application> for EffectRoutes {
    fn new(router: std::sync::Weak<EffectRouter<Application, Self>>) -> Self {
        Self {
            dns: Arc::new(Handler::new(
                Weak::clone(&router),
                SdResolver::new(SdResolverOptions {
                    service_events_callback: {
                        let router = Weak::clone(&router);
                        Arc::new(move |event| {
                            if let Some(router) = router.upgrade() {
                                router
                                    .update(Event::Core(event::CoreEvent::ServiceDiscovery(event)));
                            }
                        })
                    },
                }),
            )),
            connection: Arc::new(Handler::new(
                Weak::clone(&router),
                ExchangeResolver::new(ExchangeResolverOptions {
                    pairing_request: {
                        let router = Weak::clone(&router);

                        move |msg| {
                            let (tx, rx) = tokio::sync::oneshot::channel();

                            if let Some(router) = router.upgrade() {
                                router.update(Event::Core(event::CoreEvent::Exchange(
                                    ExchangeEvent::PairingRequest((msg, tx)),
                                )));
                            }

                            async move { rx.await.ok().unwrap_or_default() }
                        }
                    },
                }),
            )),
            storage: Arc::new(Handler::new(Weak::clone(&router), StorageResolver::new(()))),
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
                RustCoreEffect::Core(CoreEffect::ServiceDiscovery(request)) => {
                    routes.dns.process(request);
                }
                RustCoreEffect::Core(CoreEffect::Connection(request)) => {
                    routes.connection.process(request);
                }
                RustCoreEffect::Core(CoreEffect::Storage(req)) => routes.storage.process(req),
                effect => shell.process_effects(effect.into()),
            }
        });

        Self { router }
    }

    pub fn view(&self) -> ViewModel {
        self.router.view()
    }

    pub fn update(&self, event: Event) {
        self.router.update(event);
    }
}
