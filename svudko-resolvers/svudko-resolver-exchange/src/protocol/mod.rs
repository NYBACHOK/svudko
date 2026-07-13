use quinn::VarInt;

pub mod client;
pub mod server;

const EXCHANGE_FILE_CHUNK_SIZE: usize = 1024 * 1024; // 1MiB

const STREAM_PROCEED_BYTE: u8 = 0;
const STREAM_CLOSE_BYTE: u8 = 1;

const OK_STATUS: VarInt = VarInt::from_u32(0);
const PAIRING_DENIED_STATUS: VarInt = VarInt::from_u32(1);
const PAIRING_REQUIRED_GOT_NO_ID_STATUS: VarInt = VarInt::from_u32(2);

enum ClientProtocolFlow {
    Pair,
    Denied,
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Received unknown flow value")]
pub struct ClientProtocolFlowParseError;

impl TryFrom<u8> for ClientProtocolFlow {
    type Error = ClientProtocolFlowParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let res = match value {
            0 => Self::Pair,
            1 => Self::Denied,
            _ => return Err(ClientProtocolFlowParseError),
        };

        Ok(res)
    }
}

impl From<ClientProtocolFlow> for u8 {
    fn from(value: ClientProtocolFlow) -> Self {
        match value {
            ClientProtocolFlow::Pair => 0,
            ClientProtocolFlow::Denied => 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CommunicationIntent {
    Pair,
    Exchange,
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Received unknown communication intent")]
pub struct CommunicationIntentParseError;

impl TryFrom<u8> for CommunicationIntent {
    type Error = CommunicationIntentParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let res = match value {
            0 => Self::Pair,
            1 => Self::Exchange,
            _ => return Err(CommunicationIntentParseError),
        };

        Ok(res)
    }
}

impl From<CommunicationIntent> for u8 {
    fn from(value: CommunicationIntent) -> Self {
        match value {
            CommunicationIntent::Pair => 0,
            CommunicationIntent::Exchange => 1,
        }
    }
}
