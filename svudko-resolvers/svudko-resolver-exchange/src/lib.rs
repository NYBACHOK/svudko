use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr, SocketAddrV4},
};

use crux_core::capability::Operation;
use quinn::{Connection, Endpoint};
use svudko_common::{ASYNC_RUNTIME, DEFAULT_SERVER_ADDR, SERVER_PORT, resolver::HandlerResolver};

use crate::{event::ExchangeEvent, request::ExchangeCoreRequest};

mod endpoint;
pub mod event;
pub mod request;

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
}

impl From<std::io::Error> for ExchangeErrors {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct ExchangeResolver {
    endpoint: Endpoint,
    connections: HashMap<String, Connection>,
}

impl HandlerResolver for ExchangeResolver {
    type Opt = ();

    type Op = ExchangeCoreRequest;

    type Err = ExchangeErrors;

    fn new((): Self::Opt) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        Ok(Self {
            connections: HashMap::new(),
            endpoint: ASYNC_RUNTIME.block_on(crate::endpoint::endpoint(DEFAULT_SERVER_ADDR))?,
        })
    }

    async fn resolve(
        &mut self,
        op: &ExchangeCoreRequest,
    ) -> Result<<Self::Op as Operation>::Output, Self::Err> {
        match op {
            ExchangeCoreRequest::Connect((ip_addr, hostname)) => self
                .handle_connect(*ip_addr, hostname)
                .await
                .map(|()| ExchangeEvent::Connected(hostname.to_owned())),
        }
    }
}

impl ExchangeResolver {
    pub async fn handle_connect(
        &mut self,
        ip: IpAddr,
        hostname: &str,
    ) -> Result<(), ExchangeErrors> {
        let addr = match ip {
            IpAddr::V4(ip) => SocketAddr::V4(SocketAddrV4::new(ip, SERVER_PORT)),
            IpAddr::V6(_ip) => unimplemented!("ipv6 mode"), // SocketAddr::V6(SocketAddrV6::new(ip, SERVER_PORT, 0, 0)), // TODO: find real values
        };

        let connection = self.endpoint.connect(addr, "servername")?.await?;

        self.connections.insert(hostname.to_owned(), connection);

        Ok(())
    }
}
