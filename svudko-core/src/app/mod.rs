use crux_core::{
    App, Command, capability::Operation, command::NotificationBuilder, render::render,
};
use svudko_resolver_exchange::{event::ExchangeEvent, models::ClientId, request::ExchangeRequest};
use svudko_resolver_sd::{event::ServiceDiscoveryEvent, request::ServiceDiscoveryRequest};
use svudko_resolver_storage::{event::StorageEvent, request::StorageRequest};

use crate::{
    app::logic::{exchange, handle_request, sd, storage},
    event::exchange::ExchangeRequestEvent,
    view_model::LocalDevices,
};

use self::{
    effect::{CoreErrorEffect, Effect},
    event::{CoreEvent, Event},
    model::Model,
    view_model::ViewModel,
};

pub mod effect;
pub mod event;
mod logic;
pub(crate) mod model;
pub mod view_model;

#[derive(Default)]
pub(crate) struct Application;

impl App for Application {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;

    fn update(
        &self,
        event: Self::Event,
        model: &mut Self::Model,
    ) -> crux_core::Command<Self::Effect, Self::Event> {
        match event {
            Event::Initialize => Command::all([
                handle_request(StorageRequest::Fetch),
                handle_request(ServiceDiscoveryRequest::EnableService(
                    model.session_id.uuid(),
                )),
                handle_request(ServiceDiscoveryRequest::BeginBrowseForServices),
            ]),
            Event::Storage(req) => handle_request(req),
            Event::Exchange(req) => match req {
                ExchangeRequestEvent::SendFiles((hostname, files)) => {
                    let service = match model.discovered_services.get(&hostname) {
                        Some(service) => service,
                        None => {
                            return Command::notify_shell(CoreErrorEffect(
                                "failed to find such host".to_owned(),
                            ))
                            .build();
                        }
                    };

                    handle_request(ExchangeRequest::SendFiles((
                        hostname,
                        service
                            .addresses
                            .iter()
                            .find(|this| this.is_ipv4() && !this.is_loopback())
                            .unwrap()
                            .to_owned()
                            .to_ip_addr(),
                        files,
                    )))
                }
                ExchangeRequestEvent::Pair(hostname) => todo!(),
            },

            Event::Pair((hostname, is_paired)) => match model.pairing_requests.remove(&hostname) {
                Some((identifier, tx)) => {
                    let _ = tx.send(is_paired);

                    if is_paired {
                        handle_request(StorageRequest::NewHost {
                            hostname,
                            identifier,
                            overwrite: true,
                        })
                        .then(render())
                    } else {
                        render()
                    }
                }
                None => render(),
            },
            Event::Core(core_event) => match core_event {
                CoreEvent::Error(e) => {
                    tracing::error!(err = %e, "error in core");

                    Command::notify_shell(CoreErrorEffect(e)).build()
                }
                CoreEvent::ServiceDiscovery(res) => sd::handle(res, model),
                CoreEvent::Exchange(event) => exchange::handle(event, model),
                CoreEvent::Storage(event) => storage::handle(event, model),
            },
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        let view = Self::ViewModel {
            discovered_services: model
                .discovered_services
                .keys()
                .map(ToOwned::to_owned)
                .map(|hostname| LocalDevices {
                    paired: model.paired_devices.get(&hostname).is_some(),
                    hostname,
                })
                .collect(),
            pairing_requests: model
                .pairing_requests
                .iter()
                .map(|(this, _)| this.to_owned())
                .collect(),
        };

        tracing::debug!(view = ?view);

        view
    }
}
