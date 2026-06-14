use crux_core::App;

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
        model: &mut Self::Model,
    ) -> crux_core::Command<Self::Effect, Self::Event> {
        todo!()
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        todo!()
    }
}
