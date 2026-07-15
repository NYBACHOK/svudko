use std::{fs::File, io::Read, os::unix::ffi::OsStrExt, path::PathBuf};

use quinn::Connection;
use tokio::io::AsyncWriteExt;

use crate::protocol::{EXCHANGE_FILE_CHUNK_SIZE, STREAM_PROCEED_BYTE};

pub async fn handle_files_exchange_step(
    connection: &Connection,
    files: &[PathBuf],
) -> Result<(), anyhow::Error> {
    {
        let mut stream = connection.open_uni().await?;

        stream.write_u64(files.len() as u64).await?;
        stream.flush().await?;
    }

    for file in files {
        if !file.is_file() {
            continue;
        }

        let name = match file.file_name() {
            Some(name) => name,
            None => continue,
        };

        let mut stream = connection.open_uni().await?;
        stream.write_u64(name.len() as u64).await?;
        stream.write_all(name.as_bytes()).await?;

        let mut file = File::open(file)?;

        let mut buf = [0_u8; EXCHANGE_FILE_CHUNK_SIZE];
        loop {
            let size = file.read(&mut buf)?;
            if size == 0 {
                break;
            }

            stream.write_all(&buf[..size]).await?;
        }

        stream.flush().await?;
        stream.finish()?;
    }

    let mut stream = connection.open_uni().await?;
    stream.write_u8(STREAM_PROCEED_BYTE).await?;

    Ok(())
}
