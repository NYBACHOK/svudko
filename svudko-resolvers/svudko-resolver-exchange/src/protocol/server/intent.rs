use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use quinn::{Connection, RecvStream};
use svudko_common::POISONED_LOCK_MSG;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    SERVER_LOG_TAG,
    models::{ClientId, ClientIdRaw},
    protocol::{CommunicationIntent, STREAM_CLOSE_BYTE, STREAM_PROCEED_BYTE},
};

pub async fn handle_intent_exchange_step(
    connection: &Connection,
    paired_devices: Arc<RwLock<HashSet<String>>>,
) -> Result<(CommunicationIntent, Option<ClientId>), anyhow::Error> {
    let (mut send_stream, recv_stream) = connection.open_bi().await?;

    tracing::debug!(tag = %SERVER_LOG_TAG, "opened streams for intent exchange");

    // Write single byte as it required to write something before reading
    send_stream.write_u8(0).await?;

    let res = inner(recv_stream, paired_devices)
        .await
        .inspect_err(|e| tracing::error!(tag = %SERVER_LOG_TAG, err = %e ));

    if res.is_err() {
        send_stream.write_u8(STREAM_CLOSE_BYTE).await?;
    } else {
        send_stream.write_u8(STREAM_PROCEED_BYTE).await?;
    }

    send_stream.finish()?;

    res
}

async fn inner(
    mut recv_stream: RecvStream,
    paired_devices: Arc<RwLock<HashSet<String>>>,
) -> Result<(CommunicationIntent, Option<ClientId>), anyhow::Error> {
    let intent: CommunicationIntent = recv_stream.read_u8().await?.try_into()?;

    tracing::debug!(tag = %SERVER_LOG_TAG, intent = ?intent, "received intent");

    let client_id: ClientId = {
        let msg_size = recv_stream.read_u64().await? as usize;

        let buf = recv_stream
            .read_chunk(msg_size, true)
            .await?
            .ok_or_else(|| anyhow::anyhow!("stream closed before sending its identifier"))?
            .bytes;

        let archived = rkyv::access::<rkyv::Archived<ClientIdRaw>, rkyv::rancor::Error>(&buf)?;

        rkyv::deserialize::<_, rkyv::rancor::Error>(archived)?.into()
    };

    let is_paired = paired_devices
        .read()
        .expect(POISONED_LOCK_MSG)
        .contains(&client_id.id);

    tracing::debug!(tag = %SERVER_LOG_TAG, cleint_id = ?client_id, is_paired = %is_paired, "received client id");

    let client_id = match intent {
        CommunicationIntent::Pair if is_paired => None,
        CommunicationIntent::Exchange if is_paired => None,
        _ => Some(client_id),
    };

    Ok((intent, client_id))
}
