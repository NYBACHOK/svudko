use std::{
    net::{IpAddr, SocketAddr, SocketAddrV4},
    path::PathBuf,
};

use quinn::Endpoint;
use svudko_common::{SERVER_PORT, hostname::Hostname};

use crate::{
    CLIENT_LOG_TAG,
    models::{ClientId, ClientIdRaw},
    protocol::CommunicationIntent,
};

mod exchange;
mod intent;

pub async fn handle(
    endpoint: &Endpoint,
    hostname: &Hostname,
    addr: &IpAddr,
    client: ClientId,
    files: &[PathBuf],
) -> Result<Option<ClientId>, anyhow::Error> {
    let is_pairing_intent = files.is_empty();

    let addr = match *addr {
        IpAddr::V4(ip_addr) => SocketAddr::V4(SocketAddrV4::new(ip_addr, SERVER_PORT)),
        IpAddr::V6(_ip) => unimplemented!("ipv6 mode"), // SocketAddr::V6(SocketAddrV6::new(ip, SERVER_PORT, 0, 0)), // TODO: find real values
    };

    tracing::debug!(tag = %CLIENT_LOG_TAG, addr = %addr, "creating connection");

    let connection = endpoint
        .connect(addr, &hostname.to_local_dns_name())?
        .await?;

    tracing::info!(tag = %CLIENT_LOG_TAG, addr = %addr, "opened new connection");

    let client_id = ClientIdRaw::from(client);

    let server_id = intent::handle_intent_exchange_step(
        &connection,
        &client_id,
        match is_pairing_intent {
            true => CommunicationIntent::Pair,
            false => CommunicationIntent::Exchange,
        },
    )
    .await?;

    if !is_pairing_intent {
        exchange::handle_files_exchange_step(&connection, files).await?;
        return Ok(None);
    }

    let _ = connection.closed().await;

    Ok(Some(server_id))
}
