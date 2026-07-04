use quinn::Connection;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    models::ClientIdRaw,
    protocol::{CommunicationIntent, STREAM_PROCEED_BYTE},
};

pub async fn handle_intent_exchange_step(
    connection: &Connection,
    client: &ClientIdRaw,
) -> Result<(), anyhow::Error> {
    let (mut send_stream, mut recv_stream) = connection.open_bi().await?;

    send_stream
        .write_u8(CommunicationIntent::Exchange.into())
        .await?;

    {
        let client_buf = rkyv::to_bytes::<rkyv::rancor::Error>(client).expect("can't fail");

        send_stream.write_u64(client_buf.len() as u64).await?;

        send_stream.write_all(&client_buf).await?;
    }

    let is_okay = recv_stream.read_u8().await?;

    if is_okay != STREAM_PROCEED_BYTE {
        return Err(anyhow::anyhow!("server indicated error during intent step"));
    }

    Ok(())
}
