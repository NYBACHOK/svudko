use std::{io::Write, path::PathBuf};

use quinn::Connection;
use tokio::io::AsyncReadExt;

use crate::{ProtocolDescription, protocol::EXCHANGE_FILE_CHUNK_SIZE};

pub async fn handle_files_exchange_step(
    connection: &Connection,
    download_dir: PathBuf,
) -> Result<(), anyhow::Error> {
    let ProtocolDescription { files } = {
        let mut stream = connection.accept_uni().await?;

        let msg = stream.read_to_end(usize::MAX).await?;

        let archived =
            rkyv::access::<rkyv::Archived<ProtocolDescription>, rkyv::rancor::Error>(&msg)?;

        rkyv::deserialize::<_, rkyv::rancor::Error>(archived)?
    };

    for _ in files {
        let mut stream = connection.accept_uni().await?;

        let size_name = stream.read_u64().await? as usize;

        let buf = stream
            .read_chunk(size_name, true)
            .await?
            .ok_or_else(|| anyhow::anyhow!("stream closed before sending filename"))?
            .bytes;
        let filename = String::from_utf8_lossy(&buf);

        let mut file = std::fs::File::create(download_dir.join(filename.as_ref()))?;

        while let Some(chunk) = stream.read_chunk(EXCHANGE_FILE_CHUNK_SIZE, true).await? {
            file.write_all(&chunk.bytes)?;
        }

        file.flush()?;
    }

    Ok(())
}
