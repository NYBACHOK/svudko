use super::{
    Command, Effect, Event, ExchangeEvent, ExchangeRequest, Model, NotificationBuilder, Operation,
    ServiceDiscoveryEvent, StorageEvent, StorageRequest, UnknownSignature, render,
};

pub mod exchange;
pub mod sd;
pub mod storage;

/// Helper method to redirect request to rust "shell"
pub(super) fn handle_request<T: Operation>(req: T) -> crux_core::Command<Effect, Event>
where
    Effect: From<crux_core::Request<T>>,
    Event: From<<T as Operation>::Output>,
{
    Command::request_from_shell(req)
        .map(Into::into)
        .then_notify(|event| NotificationBuilder::new(async |ctx| ctx.send_event(event)))
        .build()
}
