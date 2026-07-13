use std::{
    collections::HashSet,
    fmt::Debug,
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use quinn::Endpoint;
use rcgen::SanType;
use svudko_common::{
    ASYNC_RUNTIME, DEFAULT_SERVER_ADDR, POISONED_LOCK_MSG,
    hostname::HOSTNAME,
    resolver::{HandlerResolver, Operation},
};

use crate::{
    errors::ExchangeErrors,
    event::ExchangeEvent,
    models::ClientId,
    protocol::{client, server::start_handling_incoming},
    request::ExchangeRequest,
};

pub(crate) const CLIENT_LOG_TAG: &str = "CLIENT";
pub(crate) const SERVER_LOG_TAG: &str = "SERVER";

mod endpoint;
pub mod errors;
pub mod event;
pub mod models;
mod protocol;
pub mod request;
mod verification;

#[derive(Debug)]
pub struct ExchangeResolver<T> {
    endpoint: Endpoint,
    client_id: Option<ClientId>,
    _phantom: PhantomData<T>,
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

        let _guard = ASYNC_RUNTIME.enter();

        let endpoint = crate::endpoint::endpoint(
            DEFAULT_SERVER_ADDR,
            vec![SanType::DnsName(
                HOSTNAME
                    .to_local_dns_name()
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
            paired_devices,
            client_id: None,
            _phantom: PhantomData,
        })
    }

    async fn resolve(
        &mut self,
        op: &ExchangeRequest,
    ) -> Result<<Self::Op as Operation>::Output, Self::Err> {
        match op {
            ExchangeRequest::UpdateClientId(client_id) => {
                self.client_id = Some(client_id.to_owned());

                Ok(ExchangeEvent::UpdatedClient)
            }
            ExchangeRequest::PairedDevices(trusted_hosts) => {
                *self.paired_devices.write().expect(POISONED_LOCK_MSG) = trusted_hosts.to_owned();

                Ok(ExchangeEvent::None)
            }
            ExchangeRequest::SendFiles((hostname, addr, files)) => {
                client::handle(
                    &self.endpoint,
                    hostname,
                    addr,
                    self.client_id
                        .clone()
                        .ok_or_else(|| anyhow::anyhow!("not ready to send files"))?,
                    files,
                )
                .await?;

                Ok(ExchangeEvent::None)
            }
            ExchangeRequest::Pair((hostname, addr)) => {
                client::handle(
                    &self.endpoint,
                    hostname,
                    addr,
                    self.client_id
                        .clone()
                        .ok_or_else(|| anyhow::anyhow!("not ready to send files"))?,
                    &[],
                )
                .await?;

                Ok(ExchangeEvent::None)
            }
        }
    }
}
