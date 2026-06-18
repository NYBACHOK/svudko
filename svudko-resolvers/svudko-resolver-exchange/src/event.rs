use crate::models::UnknownSignature;

#[derive(Clone, Debug)]
pub enum ExchangeEvent {
    None,
    UnknownSignature(UnknownSignature),
    // Connected(String),
    // SendFile,
}
