use std::{
    char,
    collections::{HashMap, HashSet},
    fmt::Debug,
    marker::PhantomData,
    net::{IpAddr, SocketAddr, SocketAddrV4},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use quinn::{Connection, Endpoint, Incoming, VarInt};
use rcgen::SanType;
use svudko_common::{
    ASYNC_RUNTIME, DEFAULT_SERVER_ADDR, SERVER_PORT,
    hostname::{HOSTNAME, Hostname},
    resolver::{HandlerResolver, Operation},
};
use tokio::{io::AsyncWriteExt, sync::mpsc::UnboundedReceiver};

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

#[derive(rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]

struct ProtocolDescription {
    files: Vec<String>,
}

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
            }
            ExchangeRequest::SendFiles((hostname, files)) => {
                self.send_files(hostname, files.to_vec());

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
        let connection = incoming.await?;

        let ProtocolDescription { files } = {
            let mut stream = connection.accept_uni().await?;

            let msg = stream.read_to_end(usize::MAX).await?;

            let archived =
                rkyv::access::<rkyv::Archived<ProtocolDescription>, rkyv::rancor::Error>(&msg)?;

            rkyv::deserialize::<_, rkyv::rancor::Error>(archived)?
        };

        let download_dir = dirs::download_dir().unwrap_or_default().join("svudko");
        for file in files {
            let mut file = tokio::fs::File::create_new(download_dir.join(file)).await?;

            let mut stream = connection.accept_uni().await?;

            let buf = stream.read_to_end(usize::MAX).await?;

            file.write_all(&buf).await?;
        }

        let _res = connection.closed().await;

        Ok(())
    }
}
