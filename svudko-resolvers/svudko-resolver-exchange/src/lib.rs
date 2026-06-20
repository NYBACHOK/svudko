use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    marker::PhantomData,
    net::{IpAddr, SocketAddr, SocketAddrV4},
    sync::{Arc, RwLock},
};

use quinn::{Connection, Endpoint, Incoming};
use rcgen::SanType;
use svudko_common::{
    ASYNC_RUNTIME, DEFAULT_SERVER_ADDR, SERVER_PORT,
    hostname::{HOSTNAME, Hostname},
    resolver::{HandlerResolver, Operation},
};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    errors::ExchangeErrors, event::ExchangeEvent, models::UnknownSignature,
    request::ExchangeRequest,
};

mod endpoint;
pub mod errors;
pub mod event;
pub mod models;
pub mod request;
mod verification;

const POISONED_LOCK: &str = "poisoned lock";

#[derive(Debug)]
pub struct ExchangeResolver<T> {
    endpoint: Endpoint,
    _phantom: PhantomData<T>,
    connections: HashMap<Hostname, Connection>,
    trusted_signatures: Arc<RwLock<HashSet<String>>>,
    // incoming_connections: Arc<Mutex<HashMap<String, (Connection, RecvStream)>>>,
}

pub struct ExchangeResolverOptions<T> {
    pub new_signatures_callback: T,
    // pub server_name: String,
}

impl<T> Debug for ExchangeResolverOptions<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExchangeResolverOptions").finish()
    }
}

impl<T> HandlerResolver for ExchangeResolver<T>
where
    T: Fn(UnknownSignature) + Send + Sync + 'static,
{
    type Opt = ExchangeResolverOptions<T>;

    type Op = ExchangeRequest;

    type Err = ExchangeErrors;

    fn new(
        ExchangeResolverOptions {
            new_signatures_callback,
        }: Self::Opt,
    ) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        let trusted_signatures = Default::default();

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let endpoint = ASYNC_RUNTIME.block_on(crate::endpoint::endpoint(
            DEFAULT_SERVER_ADDR,
            vec![SanType::DnsName(
                HOSTNAME
                    .as_str()
                    .try_into()
                    .expect("should be valid hostname"),
            )],
            Arc::clone(&trusted_signatures),
            tx,
        ))?;

        start_handling_new_signatures(rx, new_signatures_callback);
        start_handling_incoming(endpoint.clone());

        Ok(Self {
            endpoint,
            connections: Default::default(),
            trusted_signatures,
            _phantom: PhantomData,
        })
    }

    async fn resolve(
        &mut self,
        op: &ExchangeRequest,
    ) -> Result<<Self::Op as Operation>::Output, Self::Err> {
        match op {
            ExchangeRequest::Connect((hostname, ip_addr)) => {
                let addr = match *ip_addr {
                    IpAddr::V4(ip_addr) => SocketAddr::V4(SocketAddrV4::new(ip_addr, SERVER_PORT)),
                    IpAddr::V6(_ip) => unimplemented!("ipv6 mode"), // SocketAddr::V6(SocketAddrV6::new(ip, SERVER_PORT, 0, 0)), // TODO: find real values
                };

                let connection = self
                    .endpoint
                    .connect(addr, &hostname.to_local_dns_name())?
                    .await?;

                self.connections
                    .insert(hostname.to_owned().into(), connection);

                Ok(ExchangeEvent::None)
            }
            ExchangeRequest::TrustedSignatures(trusted_hosts) => {
                *self.trusted_signatures.write().expect(POISONED_LOCK) = trusted_hosts.to_owned();

                Ok(ExchangeEvent::None)
            } // ExchangeRequest::Send(hostname) => self
              //     .handle_send(hostname)
              //     .await
              //     .map(|()| ExchangeEvent::SendFile)
              //     .map_err(Into::into),
        }
    }
}

impl<T> ExchangeResolver<T> {
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

fn start_handling_new_signatures<T: Fn(UnknownSignature) + Send + Sync + 'static>(
    mut rx: UnboundedReceiver<UnknownSignature>,
    callback: T,
) {
    ASYNC_RUNTIME.spawn(async move {
        while let Some(msg) = rx.recv().await {
            callback(msg)
        }
    });
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
        let _connection = incoming.await?;
        tracing::info!("opened incoming connection");

        // let mut stream = connection.accept_uni().await?;

        // let mut file =
        //     tokio::fs::File::create(APP_DATA_DIR.join("received-test-image.png")).await?;

        // while let Ok(Some(chunk)) = stream.read_chunk(MAX_CHUNK_LENGTH, true).await {
        //     let _ = file.write(&chunk.bytes).await;
        // }

        // file.sync_all().await?;

        Ok(())
    }
}
