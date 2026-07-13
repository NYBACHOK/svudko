use quinn::VarInt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::models::{ClientId, ClientIdRaw};

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

async fn deserialize_client_id(
    recv_stream: &mut quinn::RecvStream,
) -> Result<ClientId, anyhow::Error> {
    let msg_size = recv_stream.read_u64().await? as usize;

    let buf = recv_stream
        .read_chunk(msg_size, true)
        .await?
        .ok_or_else(|| anyhow::anyhow!("stream closed before sending its identifier"))?
        .bytes;

    Ok(serde_json::from_slice::<ClientIdRaw>(&buf)?.into())
}

async fn serialize_client_id(
    send_stream: &mut quinn::SendStream,
    client: &ClientIdRaw,
) -> Result<(), anyhow::Error> {
    let client_buf = serde_json::to_vec(client).expect("can't fail");

    send_stream.write_u64(client_buf.len() as u64).await?;

    send_stream.write_all(&client_buf).await?;

    Ok(())
}
