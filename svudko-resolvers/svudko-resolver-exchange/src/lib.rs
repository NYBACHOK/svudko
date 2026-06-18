use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use quinn::{Connection, Endpoint, Incoming, RecvStream};
use svudko_common::{
    APP_DATA_DIR, ASYNC_RUNTIME, DEFAULT_SERVER_ADDR, SERVER_PORT,
    resolver::{HandlerResolver, Operation},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{event::ExchangeEvent, request::ExchangeRequest};

mod endpoint;
pub mod event;
pub mod request;
mod verification;

const POISONED_LOCK: &str = "poisoned lock";

const MAX_CHUNK_LENGTH: usize = 20_000_000;

#[derive(Debug, thiserror::Error)]
pub enum ExchangeErrors {
    #[error(transparent)]
    Connection(#[from] quinn::ConnectionError),
    #[error(transparent)]
    Connect(#[from] quinn::ConnectError),
    #[error("{0}")]
    Io(String),
    #[error(transparent)]
    Tls(#[from] quinn::rustls::pki_types::pem::Error),
    #[error(transparent)]
    Rustls(#[from] quinn::rustls::Error),
    #[error(transparent)]
    NoInitialCipherSuite(#[from] quinn::crypto::rustls::NoInitialCipherSuite),
    #[error(transparent)]
    Rcgen(#[from] rcgen::Error),
    #[error(transparent)]
    Tmp(#[from] anyhow::Error),
}

impl From<std::io::Error> for ExchangeErrors {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct ExchangeResolver {
    endpoint: Endpoint,
    trusted_hosts: Arc<RwLock<HashMap<String, String>>>,
    // connections: HashMap<String, Connection>,
    // incoming_connections: Arc<Mutex<HashMap<String, (Connection, RecvStream)>>>,
}

impl HandlerResolver for ExchangeResolver {
    type Opt = ();

    type Op = ExchangeRequest;

    type Err = ExchangeErrors;

    fn new((): Self::Opt) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        let trusted_hosts = Default::default();
        let endpoint = ASYNC_RUNTIME.block_on(crate::endpoint::endpoint(DEFAULT_SERVER_ADDR))?;

        start_handling_incoming(endpoint.clone());

        Ok(Self {
            trusted_hosts,
            endpoint,
        })
    }

    async fn resolve(
        &mut self,
        op: &ExchangeRequest,
    ) -> Result<<Self::Op as Operation>::Output, Self::Err> {
        match op {
            // ExchangeRequest::Connect(hostname) => {
            //     // self
            //     // .handle_connect(*ip_addr, hostname)
            //     // .await
            //     // .map(|()| ExchangeEvent::Connected(hostname.to_owned()))
            //     todo!()
            // }
            // ExchangeRequest::Send(hostname) => self
            //     .handle_send(hostname)
            //     .await
            //     .map(|()| ExchangeEvent::SendFile)
            //     .map_err(Into::into),
            ExchangeRequest::TrustedHosts(trusted_hosts) => {
                *self.trusted_hosts.write().expect(POISONED_LOCK) = trusted_hosts.to_owned();

                Ok(ExchangeEvent::None)
            }
        }
    }
}

impl ExchangeResolver {
    // pub async fn handle_connect(
    //     &mut self,
    //     ip: IpAddr,
    //     hostname: &str,
    // ) -> Result<(), ExchangeErrors> {
    //     let addr = match ip {
    //         IpAddr::V4(ip) => SocketAddr::V4(SocketAddrV4::new(ip, SERVER_PORT)),
    //         IpAddr::V6(_ip) => unimplemented!("ipv6 mode"), // SocketAddr::V6(SocketAddrV6::new(ip, SERVER_PORT, 0, 0)), // TODO: find real values
    //     };

    //     let connection = self.endpoint.connect(addr, "servername")?.await?;

    //     self.connections.insert(hostname.to_owned(), connection);

    //     Ok(())
    // }

    // pub async fn handle_send(&mut self, hostname: &str) -> Result<(), anyhow::Error> {
    //     let mut file = tokio::fs::File::open(APP_DATA_DIR.join("to-send-test-image.png")).await?;

    //     let connection = self.connections.get(hostname).unwrap();

    //     let mut stream = connection.open_uni().await?;

    //     let mut buffer = Vec::new();

    //     file.read_to_end(&mut buffer).await?;

    //     stream.write_all(&buffer).await?;

    //     Ok(())
    // }
}

fn start_handling_incoming(endpoint: Endpoint) {
    ASYNC_RUNTIME.spawn(async move {
        while let Some(accept) = endpoint.accept().await {
            let _ = handle(accept)
                .await
                .inspect_err(|e| tracing::error!(err = ?e));
        }
    });

    async fn handle(incoming: Incoming) -> Result<(), anyhow::Error> {
        let connection = incoming.await?;

        let mut stream = connection.accept_uni().await?;

        let mut file =
            tokio::fs::File::create(APP_DATA_DIR.join("received-test-image.png")).await?;

        while let Ok(Some(chunk)) = stream.read_chunk(MAX_CHUNK_LENGTH, true).await {
            let _ = file.write(&chunk.bytes).await;
        }

        file.sync_all().await?;

        Ok(())
    }
}
