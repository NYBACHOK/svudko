use crux_core::{
    App, Command, capability::Operation, command::NotificationBuilder, render::render,
};
use svudko_resolver_exchange::{
    event::ExchangeEvent, models::UnknownSignature, request::ExchangeRequest,
};
use svudko_resolver_sd::event::ServiceDiscoveryEvent;
use svudko_resolver_storage::{event::StorageEvent, request::StorageRequest};

use crate::{
    app::logic::{exchange, handle_request, sd, storage},
    event::exchange::ExchangeRequestEvent,
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
pub struct Application;

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
            Event::Initialize => Command::all([handle_request(StorageRequest::Fetch)]),
            Event::ServiceDiscovery(req) => handle_request(req),
            Event::Storage(req) => handle_request(req),
            Event::Exchange(req) => match req {
                ExchangeRequestEvent::Connect(hostname) => {
                    match model
                        .dns_sd
                        .discovered_services
                        .get(&hostname.to_local_dns_name())
                    {
                        Some(service) => handle_request(ExchangeRequest::Connect((
                            hostname,
                            service
                                .addresses
                                .iter()
                                .find(|this| this.is_ipv4() && !this.is_loopback())
                                .unwrap()
                                .to_owned()
                                .to_ip_addr(),
                        ))),
                        None => Command::notify_shell(CoreErrorEffect(
                            "failed to find such host".to_owned(),
                        ))
                        .build(),
                    }
                }
                ExchangeRequestEvent::SendFiles(files) => {
                    handle_request(ExchangeRequest::SendFiles(files))
                }
            },
            Event::AllowHost((hostname, signature)) => handle_request(StorageRequest::NewHost {
                hostname,
                signature,
                overwrite: false,
            }),
            Event::Core(core_event) => match core_event {
                CoreEvent::Error(e) => {
                    tracing::error!(err = %e, "error in core");

                    Command::notify_shell(CoreErrorEffect(e)).build()
                }
                CoreEvent::DnsReponses(res) => sd::handle(res, model),
                CoreEvent::Exchange(event) => exchange::handle(event, model),
                CoreEvent::Storage(event) => storage::handle(event, model),
            },
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        let view = Self::ViewModel {
            enabled_discover: model.dns_sd.enabled_discover,
            unknown_signatures: model
                .unknown_signatures
                .clone()
                .into_iter()
                .map(|(host, signature)| (host.into(), signature))
                .collect(),
            discovered_services: model
                .dns_sd
                .discovered_services
                .keys()
                .map(ToOwned::to_owned)
                .collect(),
        };

        tracing::debug!(view = ?view);

        view
    }
}
