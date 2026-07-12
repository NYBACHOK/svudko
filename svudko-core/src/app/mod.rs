use crux_core::{
    App, Command, capability::Operation, command::NotificationBuilder, render::render,
};
use svudko_resolver_exchange::{event::ExchangeEvent, models::ClientId, request::ExchangeRequest};
use svudko_resolver_sd::{event::ServiceDiscoveryEvent, request::ServiceDiscoveryRequest};
use svudko_resolver_storage::{event::StorageEvent, request::StorageRequest};

use crate::{app::logic::handle_request, view_model::LocalDevices};

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
                handle_request(StorageRequest::ClientId),
                handle_request(ServiceDiscoveryRequest::EnableService(
                    model.session_id.uuid(),
                )),
                handle_request(ServiceDiscoveryRequest::BeginBrowseForServices),
            ]),
            Event::Storage(req) => handle_request(req),
            Event::Exchange(req) => event::exchange::handle(req, model),
            Event::Core(core_event) => match core_event {
                CoreEvent::Error(e) => {
                    tracing::error!(err = %e, "error in core");

                    Command::notify_shell(CoreErrorEffect(e)).build()
                }
                CoreEvent::ServiceDiscovery(res) => logic::sd::handle(res, model),
                CoreEvent::Exchange(event) => logic::exchange::handle(event, model),
                CoreEvent::Storage(event) => logic::storage::handle(event, model),
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
