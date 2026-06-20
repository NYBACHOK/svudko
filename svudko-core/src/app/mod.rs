use crux_core::{
    App, Command, capability::Operation, command::NotificationBuilder, render::render,
};
use svudko_resolver_exchange::{
    event::ExchangeEvent, models::UnknownSignature, request::ExchangeRequest,
};
use svudko_resolver_sd::event::ServiceDiscoveryEvent;
use svudko_resolver_storage::{event::StorageEvent, request::StorageRequest};

pub mod effect;
pub mod event;
pub(crate) mod model;
pub mod view_model;

use crate::event::exchange::ExchangeRequestEvent;

use self::{
    effect::{CoreErrorEffect, Effect},
    event::{CoreEvent, Event},
    model::Model,
    view_model::ViewModel,
};

fn handle_request<T: Operation>(req: T) -> crux_core::Command<Effect, Event>
where
    Effect: From<crux_core::Request<T>>,
    Event: From<<T as Operation>::Output>,
{
    Command::request_from_shell(req)
        .map(Into::into)
        .then_notify(|event| NotificationBuilder::new(async |ctx| ctx.send_event(event)))
        .build()
}

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
                ExchangeRequestEvent::SendFiles(files ) => {
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
                CoreEvent::DnsReponses(res) => handle_dns_events(res, model),
                CoreEvent::Exchange(event) => handle_quick_events(event, model),
                CoreEvent::Storage(event) => handle_storage_events(event, model),
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

fn handle_storage_events(
    event: StorageEvent,
    model: &mut Model,
) -> crux_core::Command<Effect, Event> {
    match event {
        StorageEvent::Fetch(trusted_hosts) => {
            model.trusted_signatures = trusted_hosts
                .into_iter()
                .map(|this| this.signature)
                .collect();

            model.load_state.hosts = true;

            handle_request(ExchangeRequest::TrustedSignatures(
                model.trusted_signatures.clone(),
            ))
            .then(Command::done())
        }
        StorageEvent::HostAlreadyExists(host) => {
            let _ = model.unknown_signatures.remove(&host);

            handle_request(StorageRequest::Fetch).then(render())
        }
        StorageEvent::HostAdded(host) => {
            let _ = model.unknown_signatures.remove(&host.hostname);

            handle_request(StorageRequest::Fetch).then(render())
        }
    }
}

fn handle_quick_events(
    event: ExchangeEvent,
    model: &mut Model,
) -> crux_core::Command<Effect, Event> {
    match event {
        ExchangeEvent::None => Command::done(),
        ExchangeEvent::UnknownSignature(UnknownSignature {
            hostname,
            signature,
        }) => {
            model.unknown_signatures.insert(hostname, signature);
            render()
        }
    }
}

fn handle_dns_events(
    event: ServiceDiscoveryEvent,
    model: &mut Model,
) -> crux_core::Command<Effect, Event> {
    tracing::debug!(method = "handle_dns_events", event = ?event);

    match event {
        ServiceDiscoveryEvent::Enabled => model.dns_sd.enabled_discover = true,
        ServiceDiscoveryEvent::Disabled => model.dns_sd.enabled_discover = false,
        ServiceDiscoveryEvent::FoundServices(services) => {
            model.dns_sd.discovered_services = services
        }
        ServiceDiscoveryEvent::FoundIps(_ips) => todo!(),
    }

    render()
}
