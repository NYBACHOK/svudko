use svudko_resolver_storage::request::StorageRequest;

use crate::app::handle_request;

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
        ExchangeEvent::PairedWithServer(ClientId { hostname, id }) => {
            handle_request(StorageRequest::NewHost {
                hostname,
                identifier: id,
                overwrite: true,
            })
        }
        ExchangeEvent::UpdatedClient => {
            model.load_state.client_id = true;

            Command::done()
        }
    }
}
