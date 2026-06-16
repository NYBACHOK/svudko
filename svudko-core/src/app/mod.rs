use crux_core::{App, Command, command::NotificationBuilder, render::render};

pub mod effect;
pub mod event;
mod model;
pub mod view_model;

use self::{effect::*, event::*, model::*, view_model::*};

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
        _model: &mut Self::Model,
    ) -> crux_core::Command<Self::Effect, Self::Event> {
        match event {
            Event::Dns(req) => Command::request_from_shell(req)
                .map(|this| match this {
                    Ok(res) => Event::Core(CoreEvent::DnsReponses(res)),
                    Err(err) => Event::Core(CoreEvent::Error(err.to_string())),
                })
                .then_notify(|event| NotificationBuilder::new(async |ctx| ctx.send_event(event)))
                .build(),

            Event::Core(core_event) => match core_event {
                CoreEvent::DnsReponses(res) => {
                    tracing::info!("dns response: {res:#?}");

                    render()
                }
                CoreEvent::Error(e) => {
                    tracing::error!(err = %e, "error in core");

                    Command::notify_shell(CoreErrorEffect(e)).build()
                }
            },
        }
    }

    fn view(&self, _model: &Self::Model) -> Self::ViewModel {
        Self::ViewModel {}
    }
}
