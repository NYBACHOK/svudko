use super::*;

pub fn handle(event: ExchangeEvent, model: &mut Model) -> crux_core::Command<Effect, Event> {
    match event {
        ExchangeEvent::None => Command::done(),
        ExchangeEvent::UnknownSignature(UnknownSignature {
            hostname,
            signature,
        }) => {
            model.unknown_signatures.insert(hostname, signature);
            render()
        }
    }
}
