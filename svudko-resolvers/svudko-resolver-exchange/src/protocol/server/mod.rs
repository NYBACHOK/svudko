use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use quinn::{Endpoint, Incoming};
use svudko_common::ASYNC_RUNTIME;

use crate::{
    models::ClientId,
    protocol::{OK_STATUS, PAIRING_DENIED_STATUS, PAIRING_REQUIRED_GOT_NO_ID_STATUS},
};

mod exchange;
mod intent;

pub fn start_handling_incoming<T, U>(
    endpoint: Endpoint,
    paired_devices: Arc<RwLock<HashSet<String>>>,
    pairing_handle: Arc<T>,
) where
    T: Fn(ClientId) -> U + Sync + Send + 'static,
    U: Future<Output = bool> + Send + Sync + 'static,
{
    ASYNC_RUNTIME.spawn(async move {
        let download_dir = dirs::download_dir().unwrap_or_default().join("svudko");
        if !download_dir.exists() {
            let _ = std::fs::create_dir(&download_dir);
        }

        while let Some(accept) = endpoint.accept().await {
            let _ = handle(
                accept,
                Arc::clone(&paired_devices),
                download_dir.clone(),
                Arc::clone(&pairing_handle),
            )
            .await
            .inspect_err(|e| tracing::error!(err = ?e));
        }
    });
}

async fn handle<T, U>(
    incoming: Incoming,
    paired_devices: Arc<RwLock<HashSet<String>>>,
    download_dir: PathBuf,
    pairing_handle: Arc<T>,
) -> Result<(), anyhow::Error>
where
    T: Fn(ClientId) -> U,
    U: Future<Output = bool> + Send + Sync + 'static,
{
    let connection = incoming.await?;

    let (intent, signature) =
        intent::handle_intent_exchange_step(&connection, paired_devices).await?;

    match (intent, signature) {
        (super::CommunicationIntent::Pair, None) => connection.close(OK_STATUS, b"paired"),
        (super::CommunicationIntent::Pair, Some(signature)) => {
            let is_paired = pairing_handle(signature).await;

            if is_paired {
                connection.close(OK_STATUS, b"paired")
            } else {
                connection.close(PAIRING_DENIED_STATUS, b"pairing denied")
            }
        }
        (super::CommunicationIntent::Exchange, None) => {
            connection.close(PAIRING_REQUIRED_GOT_NO_ID_STATUS, b"need pairing")
        }
        (super::CommunicationIntent::Exchange, Some(_)) => {
            exchange::handle_files_exchange_step(&connection, download_dir).await?
        }
    }

    Ok(())
}
