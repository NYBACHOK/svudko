use crux_core::{App, Command, command::NotificationBuilder, render::render};
use svudko_resolver_exchange::{event::ExchangeEvent, request::ExchangeCoreRequest};
use svudko_resolver_sd::event::ServiceDiscoveryEvent;

pub mod effect;
pub mod event;
mod model;
pub mod view_model;

use self::{
    effect::{CoreErrorEffect, Effect},
    event::{CoreEvent, Event, ExchangeRequest},
    model::Model,
    view_model::ViewModel,
};

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
            Event::Dns(req) => Command::request_from_shell(req)
                .map(|this| Event::Core(CoreEvent::DnsReponses(this)))
                .then_notify(|event| NotificationBuilder::new(async |ctx| ctx.send_event(event)))
                .build(),
            Event::Exchange(ExchangeRequest::Connect(hostname)) => {
                let addr = model
                    .dns_sd
                    .discovered_services
                    .get(&hostname)
                    .cloned()
                    .unwrap()
                    .addresses
                    .iter()
                    .find(|this| this.is_ipv4())
                    .unwrap()
                    .to_ip_addr();

                Command::request_from_shell(ExchangeCoreRequest::Connect((addr, hostname)))
                    .map(|this| Event::Core(CoreEvent::Exchange(this)))
                    .then_notify(|event| {
                        NotificationBuilder::new(async |ctx| ctx.send_event(event))
                    })
                    .build()
            },
            Event::Exchange(ExchangeRequest::SendFile(hostname)) =>   Command::request_from_shell(ExchangeCoreRequest::Send(hostname))
                    .map(|this| Event::Core(CoreEvent::Exchange(this)))
                    .then_notify(|event| {
                        NotificationBuilder::new(async |ctx| ctx.send_event(event))
                    })
                    .build(),
            Event::Core(core_event) => match core_event {
                CoreEvent::Error(e) => {
                    tracing::error!(err = %e, "error in core");

                    Command::notify_shell(CoreErrorEffect(e)).build()
                }
                CoreEvent::DnsReponses(res) => handle_dns_events(res, model),
                CoreEvent::Exchange(event) => handle_quick_events(event, model),
            },
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        let view = Self::ViewModel {
            enabled_discover: model.dns_sd.enabled_discover,
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

fn handle_quick_events(
    event: ExchangeEvent,
    model: &mut Model,
) -> crux_core::Command<Effect, Event> {
    match event {
        ExchangeEvent::Connected(hostname) => {
            let _ = model.connected.insert(hostname);
        }
        ExchangeEvent::SendFile => (),
    }

    render()
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
        ServiceDiscoveryEvent::FoundIps(ips) => model.dns_sd.dedicated_search = ips,
    }

    render()
}
