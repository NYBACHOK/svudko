use std::{
    net::{IpAddr, SocketAddr, SocketAddrV4},
    path::PathBuf,
};

use quinn::Endpoint;
use svudko_common::{SERVER_PORT, hostname::Hostname};

use crate::{
    models::{ClientId, ClientIdRaw},
    protocol::CommunicationIntent,
};

mod exchange;
mod intent;

pub async fn handle_exchange(
    endpoint: &Endpoint,
    hostname: &Hostname,
    addr: &IpAddr,
    client: ClientId,
    files: &[PathBuf],
) -> Result<(), anyhow::Error> {
    let addr = match *addr {
        IpAddr::V4(ip_addr) => SocketAddr::V4(SocketAddrV4::new(ip_addr, SERVER_PORT)),
        IpAddr::V6(_ip) => unimplemented!("ipv6 mode"), // SocketAddr::V6(SocketAddrV6::new(ip, SERVER_PORT, 0, 0)), // TODO: find real values
    };

    let connection = endpoint.connect(addr, hostname.as_str())?.await?;

    let client_id = ClientIdRaw::from(client);

    intent::handle_intent_exchange_step(
        &connection,
        &client_id,
        CommunicationIntent::Exchange.into(),
    )
    .await?;

    exchange::handle_files_exchange_step(&connection, files).await?;

    Ok(())
}

pub async fn handle_pair(
    endpoint: &Endpoint,
    hostname: &Hostname,
    addr: &IpAddr,
    client: ClientId,
) -> Result<(), anyhow::Error> {
    let addr = match *addr {
        IpAddr::V4(ip_addr) => SocketAddr::V4(SocketAddrV4::new(ip_addr, SERVER_PORT)),
        IpAddr::V6(_ip) => unimplemented!("ipv6 mode"), // SocketAddr::V6(SocketAddrV6::new(ip, SERVER_PORT, 0, 0)), // TODO: find real values
    };

    let connection = endpoint
        .connect(addr, &hostname.to_local_dns_name())?
        .await?;

    let client_id = ClientIdRaw::from(client);

    intent::handle_intent_exchange_step(&connection, &client_id, CommunicationIntent::Pair.into())
        .await?;

    Ok(())
}
