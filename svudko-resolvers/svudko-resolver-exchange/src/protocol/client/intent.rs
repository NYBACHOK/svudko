use quinn::Connection;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    CLIENT_LOG_TAG,
    models::{ClientId, ClientIdRaw},
    protocol::{
        CommunicationIntent, STREAM_PROCEED_BYTE, deserialize_client_id, serialize_client_id,
    },
};

pub async fn handle_intent_exchange_step(
    connection: &Connection,
    client: &ClientIdRaw,
    intent: CommunicationIntent,
) -> Result<ClientId, anyhow::Error> {
    let (mut send_stream, mut recv_stream) = connection.open_bi().await?;

    tracing::debug!(tag = %CLIENT_LOG_TAG, "opened streams for intent exchange");

    send_stream.write_u8(intent.into()).await?;

    tracing::debug!(tag = %CLIENT_LOG_TAG, intent = ?intent, "send intent");

    serialize_client_id(&mut send_stream, client).await?;

    send_stream.flush().await?;

    let is_okay = recv_stream.read_u8().await?;

    if is_okay != STREAM_PROCEED_BYTE {
        tracing::error!(tag = %CLIENT_LOG_TAG, "server signaled to close connection during intens exchange");
        return Err(anyhow::anyhow!("server indicated error during intent step"));
    }

    let client_id = deserialize_client_id(&mut recv_stream).await?;

    // todo: verification of server by client too

    Ok(client_id)
}
