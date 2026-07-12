use std::path::PathBuf;

use svudko_common::hostname::Hostname;
use svudko_resolver_exchange::request::ExchangeRequest;
use svudko_resolver_storage::request::StorageRequest;

use crate::{app::handle_request, effect::Effect, event::Event, model::Model};

#[derive(Clone, Debug)]
pub enum ExchangeRequestEvent {
    PairRequest(Hostname),
    PairResponse((Hostname, bool)),
    SendFiles((Hostname, Vec<PathBuf>)),
}

pub(crate) fn handle(
    req: ExchangeRequestEvent,
    model: &mut Model,
) -> crux_core::Command<Effect, Event> {
    match req {
        ExchangeRequestEvent::SendFiles((hostname, files)) => {
            let addr = model.addr_for_discovered_device(&hostname).unwrap();

            handle_request(ExchangeRequest::SendFiles((hostname, addr, files)))
        }
        ExchangeRequestEvent::PairRequest(hostname) => {
            let addr = model.addr_for_discovered_device(&hostname).unwrap();

            handle_request(ExchangeRequest::Pair((hostname, addr)))
        }
        ExchangeRequestEvent::PairResponse((hostname, is_pair)) => {
            let (identifier, handler) = model.pairing_requests.remove(&hostname).unwrap();

            let _ = handler.send(is_pair);

            handle_request(StorageRequest::NewHost {
                hostname,
                identifier,
                overwrite: true,
            })
        }
    }
}
