use std::{
    char,
    collections::{HashMap, HashSet},
    fmt::Debug,
    marker::PhantomData,
    net::{IpAddr, SocketAddr, SocketAddrV4},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use quinn::{Connection, Endpoint, VarInt};
use rcgen::SanType;
use svudko_common::{
    ASYNC_RUNTIME, DEFAULT_SERVER_ADDR, POISONED_LOCK_MSG, SERVER_PORT,
    hostname::{HOSTNAME, Hostname},
    resolver::{HandlerResolver, Operation},
};

use crate::{
    errors::ExchangeErrors, event::ExchangeEvent, models::ClientId,
    protocol::server::start_handling_incoming, request::ExchangeRequest,
};

mod endpoint;
pub mod errors;
pub mod event;
pub mod models;
mod protocol;
pub mod request;
mod verification;

#[derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
struct ProtocolDescription {
    files: Vec<String>,
}

#[derive(Debug)]
pub struct ExchangeResolver<T> {
    endpoint: Endpoint,
    _phantom: PhantomData<T>,
    connections: HashMap<Hostname, Connection>,
    paired_devices: Arc<RwLock<HashSet<String>>>,
    // incoming_connections: Arc<Mutex<HashMap<String, (Connection, RecvStream)>>>,
}

pub struct ExchangeResolverOptions<T> {
    pub pairing_request: T,
    // pub server_name: String,
}

impl<T> Debug for ExchangeResolverOptions<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExchangeResolverOptions").finish()
    }
}

impl<T, U> HandlerResolver for ExchangeResolver<T>
where
    T: Fn(ClientId) -> U + Send + Sync + 'static,
    U: Future<Output = bool> + Send + Sync + 'static,
{
    type Opt = ExchangeResolverOptions<T>;

    type Op = ExchangeRequest;

    type Err = ExchangeErrors;

    fn new(ExchangeResolverOptions { pairing_request }: Self::Opt) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        let paired_devices = Default::default();

        let endpoint = crate::endpoint::endpoint(
            DEFAULT_SERVER_ADDR,
            vec![SanType::DnsName(
                HOSTNAME
                    .as_str()
                    .try_into()
                    .expect("should be valid hostname"),
            )],
        )?;

        start_handling_incoming(
            endpoint.clone(),
            Arc::clone(&paired_devices),
            Arc::new(pairing_request),
        );

        Ok(Self {
            endpoint,
            connections: Default::default(),
            paired_devices,
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

                self.connections.insert(hostname.to_owned(), connection);

                Ok(ExchangeEvent::None)
            }
            ExchangeRequest::PairedDevices(trusted_hosts) => {
                *self.paired_devices.write().expect(POISONED_LOCK_MSG) = trusted_hosts.to_owned();

                Ok(ExchangeEvent::None)
            }
            ExchangeRequest::SendFiles((hostname, files)) => {
                self.send_files(hostname, files.clone());

                Ok(ExchangeEvent::None)
            }
        }
    }
}

impl<T> ExchangeResolver<T> {
    pub fn send_files(&mut self, hostname: &Hostname, files: Vec<PathBuf>) {
        let connection = self.connections.remove(hostname).unwrap();

        ASYNC_RUNTIME.spawn(async move {
            let _ = handle(connection, files)
                .await
                .inspect_err(|e| tracing::error!(err = %e, "during files sending"));
        });

        async fn handle(connection: Connection, files: Vec<PathBuf>) -> Result<(), anyhow::Error> {
            {
                let description = ProtocolDescription {
                    files: files
                        .iter()
                        .filter_map(|this| {
                            this.file_name().map(|this| {
                                this.to_string_lossy()
                                    .replace(char::REPLACEMENT_CHARACTER, "")
                            })
                        })
                        .collect(),
                };

                let desc_buf = rkyv::to_bytes::<rkyv::rancor::Error>(&description)?;

                let mut stream = connection.open_uni().await?;

                stream.write_all(desc_buf.as_slice()).await?;

                let _ = stream.finish();

                tracing::info!("wrote description");
            }

            for file_path in files {
                let mut stream = connection.open_uni().await?;

                tracing::info!("writing file: {}", file_path.display());

                let file = tokio::fs::read(file_path).await?;

                stream.write_all(&file).await?;

                let _ = stream.finish();
                stream.stopped().await?;
            }

            connection.close(VarInt::from_u32(0), "finish".as_bytes());

            Ok(())
        }
    }
}
