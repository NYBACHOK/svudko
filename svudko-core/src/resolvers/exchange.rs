use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

use svudko_common::{
    DEFAULT_SERVER_ADDR, SERVER_PORT,
    quinn::{self, Connection, Endpoint},
};

use crate::{TOKIO_RUNTIME, effect::ExchangeCoreRequest, event::ExchangeEvent};

use super::*;

#[derive(Debug, thiserror::Error)]
pub enum ExchangeErrors {
    #[error(transparent)]
    Connection(#[from] quinn::ConnectionError),
    #[error(transparent)]
    Connect(#[from] quinn::ConnectError),
    #[error(transparent)]
    Tmp(#[from] anyhow::Error),
}

#[derive(Clone)]
pub struct Resolver {
    endpoint: Endpoint,
    connections: HashMap<String, Connection>,
}

impl Resolver {
    pub fn new() -> Result<Self, ExchangeErrors> {
        Ok(Self {
            connections: HashMap::new(),
            endpoint: TOKIO_RUNTIME
                .block_on(svudko_common::endpoint::endpoint(DEFAULT_SERVER_ADDR))?,
        })
    }
}

impl HandlerResolver<ExchangeCoreRequest, <ExchangeCoreRequest as Operation>::Output> for Resolver {
    async fn resolve(
        &mut self,
        op: &ExchangeCoreRequest,
    ) -> <ExchangeCoreRequest as Operation>::Output {
        let event = match op {
            ExchangeCoreRequest::Connect((ip_addr, hostname)) => self
                .handle_connect(*ip_addr, hostname)
                .await
                .map(|_| ExchangeEvent::Connected(hostname.to_owned())),
        };

        event
    }
}

impl Resolver {
    async fn handle_connect(&mut self, ip: IpAddr, hostname: &str) -> Result<(), ExchangeErrors> {
        let addr = match ip {
            IpAddr::V4(ip) => SocketAddr::V4(SocketAddrV4::new(ip, SERVER_PORT)),
            IpAddr::V6(ip) => SocketAddr::V6(SocketAddrV6::new(ip, SERVER_PORT, 0, 0)), // TODO: find real values
        };

        let connection = self.endpoint.connect(addr, "servername")?.await?;

        self.connections.insert(hostname.to_owned(), connection);

        Ok(())
    }
}
