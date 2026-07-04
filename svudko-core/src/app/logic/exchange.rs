use super::{ClientId, Command, Effect, Event, ExchangeEvent, Model, render};

pub fn handle(event: ExchangeEvent, model: &mut Model) -> crux_core::Command<Effect, Event> {
    match event {
        ExchangeEvent::None => Command::done(),
        ExchangeEvent::PairingRequest((
            ClientId {
                hostname,
                id: signature,
            },
            handler,
        )) => {
            let _ = model
                .pairing_requests
                .insert(hostname, (signature, handler));

            render()
        }
        ExchangeEvent::UpdatedClient => {
            model.load_state.client_id = true;

            Command::done()
        }
    }
}
