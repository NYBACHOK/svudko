use crux_core::{App, Command, command::NotificationBuilder, render::render};

mod effect;
mod event;
mod model;
mod view_model;

pub use self::{effect::*, event::*, model::*, view_model::*};

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
            Event::Error(e) => {
                tracing::error!(err = %e, "error in core");

                render()
            }
            Event::Dns(req) => Command::request_from_shell(req)
                .map(|this| match this {
                    Ok(res) => Event::DnsReponses(res),
                    Err(err) => Event::Error(err.to_string()),
                })
                .then_notify(|event| NotificationBuilder::new(async |ctx| ctx.send_event(event)))
                .build(),
            Event::DnsReponses(res) => {
                tracing::info!("dns response: {res:#?}");

                render()
            }
        }
    }

    fn view(&self, _model: &Self::Model) -> Self::ViewModel {
        Self::ViewModel {}
    }
}
