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
    }
}
