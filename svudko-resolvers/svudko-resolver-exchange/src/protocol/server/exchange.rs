use std::{io::Write, path::PathBuf};

use quinn::Connection;
use tokio::io::AsyncReadExt;

use crate::{SERVER_LOG_TAG, protocol::EXCHANGE_FILE_CHUNK_SIZE};

pub async fn handle_files_exchange_step(
    connection: &Connection,
    download_dir: PathBuf,
) -> Result<(), anyhow::Error> {
    let files_num = {
        let mut stream = connection.accept_uni().await?;

        stream.read_u64().await?
    };

    tracing::debug!(tag = %SERVER_LOG_TAG, files_num = %files_num, "received number of files");

    for _ in 0..files_num {
        let mut stream = connection.accept_uni().await?;

        let size_name = stream.read_u64().await? as usize;

        let buf = stream
            .read_chunk(size_name, true)
            .await?
            .ok_or_else(|| anyhow::anyhow!("stream closed before sending filename"))?
            .bytes;
        let filename = String::from_utf8_lossy(&buf);

        tracing::debug!(tag = %SERVER_LOG_TAG, filename = %filename, "receiving file");

        let mut file = std::fs::File::create(download_dir.join(filename.as_ref()))?;

        while let Some(chunk) = stream.read_chunk(EXCHANGE_FILE_CHUNK_SIZE, true).await? {
            file.write_all(&chunk.bytes)?;
        }

        file.flush()?;

        tracing::debug!(tag = %SERVER_LOG_TAG, filename = %filename, "saved file");
    }

    let mut stream = connection.accept_uni().await?;
    let _ = stream.read_u8().await?;

    Ok(())
}
