use super::{
    Command, Effect, Event, ExchangeRequest, Model, StorageEvent, StorageRequest, handle_request,
    render,
};

pub fn handle(event: StorageEvent, model: &mut Model) -> crux_core::Command<Effect, Event> {
    match event {
        StorageEvent::Fetch(trusted_hosts) => {
            model.paired_devices = trusted_hosts
                .into_iter()
                .map(|this| (this.hostname, this.signature))
                .collect();

            model.load_state.hosts = true;

            handle_request(ExchangeRequest::TrustedSignatures(
                model.paired_devices.values().cloned().collect(),
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
