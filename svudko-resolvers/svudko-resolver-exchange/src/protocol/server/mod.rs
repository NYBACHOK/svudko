use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use quinn::{Endpoint, Incoming};
use svudko_common::{ASYNC_RUNTIME, POISONED_LOCK_MSG};

use crate::{
    SERVER_LOG_TAG,
    models::ClientId,
    protocol::{OK_STATUS, PAIRING_DENIED_STATUS, PAIRING_REQUIRED_GOT_NO_ID_STATUS},
};

mod exchange;
mod intent;

pub fn start_handling_incoming<T, U>(
    endpoint: Endpoint,
    paired_devices: Arc<RwLock<HashSet<String>>>,
    pairing_handle: Arc<T>,
    client: Arc<RwLock<Option<ClientId>>>,
) where
    T: Fn(ClientId) -> U + Sync + Send + 'static,
    U: Future<Output = bool> + Send + Sync + 'static,
{
    ASYNC_RUNTIME.spawn(async move {
        let download_dir = dirs::download_dir().unwrap_or_default().join("svudko");
        if !download_dir.exists() {
            let _ = std::fs::create_dir(&download_dir).inspect_err(|e| tracing::error!(tag = %SERVER_LOG_TAG, err = ?e));
        }

        while let Some(incoming) = endpoint.accept().await {
            tracing::info!(tag = %SERVER_LOG_TAG, addr = %incoming.remote_address(), "received incoming connection");

            let client = match client.read().expect(POISONED_LOCK_MSG).clone() {
                Some(client) => client,
                None => continue,
            };

            let _ = handle(
                incoming,
                Arc::clone(&paired_devices),
                download_dir.clone(),
                Arc::clone(&pairing_handle),
                client,
            )
            .await
            .inspect_err(|e| tracing::error!(tag = %SERVER_LOG_TAG, err = ?e));
        }
    });
}

async fn handle<T, U>(
    incoming: Incoming,
    paired_devices: Arc<RwLock<HashSet<String>>>,
    download_dir: PathBuf,
    pairing_handle: Arc<T>,
    client: ClientId,
) -> Result<(), anyhow::Error>
where
    T: Fn(ClientId) -> U,
    U: Future<Output = bool> + Send + Sync + 'static,
{
    let connection = incoming.await?;

    tracing::info!(tag = %SERVER_LOG_TAG, addr = %connection.remote_address(), "opened new connection");

    let (intent, signature) =
        intent::handle_intent_exchange_step(&connection, paired_devices, client).await?;

    tracing::debug!(tag = %SERVER_LOG_TAG,  intent =?intent, addr = %connection.remote_address(), is_paired = %signature.is_some(), "exchanged intents");

    match (intent, signature) {
        (super::CommunicationIntent::Pair, None) => connection.close(OK_STATUS, b"paired"),
        (super::CommunicationIntent::Pair, Some(signature)) => {
            let is_paired = pairing_handle(signature).await;

            if is_paired {
                connection.close(OK_STATUS, b"paired");
            } else {
                connection.close(PAIRING_DENIED_STATUS, b"pairing denied");
            }
        }
        (super::CommunicationIntent::Exchange, None) => {
            connection.close(PAIRING_REQUIRED_GOT_NO_ID_STATUS, b"need pairing")
        }
        (super::CommunicationIntent::Exchange, Some(_)) => {
            exchange::handle_files_exchange_step(&connection, download_dir).await?;

            connection.close(OK_STATUS, b"received all files");
        }
    }

    Ok(())
}
