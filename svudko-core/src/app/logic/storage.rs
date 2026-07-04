use svudko_common::hostname::HOSTNAME;
use svudko_resolver_exchange::models::ClientId;

use super::{
    Command, Effect, Event, ExchangeRequest, Model, StorageEvent, StorageRequest, handle_request,
    render,
};

pub fn handle(event: StorageEvent, model: &mut Model) -> crux_core::Command<Effect, Event> {
    match event {
        StorageEvent::Fetch(trusted_hosts) => {
            model.paired_devices = trusted_hosts
                .into_iter()
                .map(|this| (this.hostname, this.identifier))
                .collect();

            model.load_state.paired_devices = true;

            handle_request(ExchangeRequest::PairedDevices(
                model.paired_devices.values().cloned().collect(),
            ))
            .then(Command::done())
        }
        StorageEvent::DeviceAlreadyExists(_) => {
            handle_request(StorageRequest::Fetch).then(render())
        }
        StorageEvent::DeviceAdded(_) => handle_request(StorageRequest::Fetch).then(render()),
        StorageEvent::ClientId(uuid) => {
            let client = ClientId {
                hostname: HOSTNAME.to_owned(),
                id: uuid.to_string(),
            };

            handle_request(ExchangeRequest::UpdateClientId(client))
        }
    }
}
