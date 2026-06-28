use crate::models::ClientId;

#[derive(Debug)]
pub enum ExchangeEvent {
    None,
    PairingRequest((ClientId, tokio::sync::oneshot::Sender<bool>)),
    // Connected(String),
    // SendFile,
}
