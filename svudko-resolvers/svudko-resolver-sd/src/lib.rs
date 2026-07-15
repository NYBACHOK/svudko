use std::{collections::HashSet, sync::Arc, time::Duration};

use mdns_sd::{HostnameResolutionEvent, ServiceDaemon, ServiceInfo};
use svudko_common::{
    ASYNC_RUNTIME,
    hostname::{HOSTNAME, Hostname},
    resolver::{HandlerResolver, Operation},
};

use crate::{
    event::ServiceDiscoveryEvent,
    models::{LocalService, ScopedIp},
    request::ServiceDiscoveryRequest,
};

pub mod event;
pub mod models;
pub mod request;

const OPERATION_TIMEOUT: Duration = Duration::from_secs(20);

pub const MDNS_SERVICE_PORT: u16 = 15571;
pub const MDNS_SERVICE_TYPE: &str = "_svudko-app._udp.local.";

#[derive(Clone, Debug, thiserror::Error)]
pub enum ServiceDiscoveryErrors {
    #[error(transparent)]
    Mdns(#[from] mdns_sd::Error),
}

#[derive(Clone)]
pub struct SdResolverOptions<T> {
    pub service_events_callback: Arc<T>,
}

impl<T> std::fmt::Debug for SdResolverOptions<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExchangeResolverOptions").finish()
    }
}

#[derive(Clone)]
pub struct SdResolver<T> {
    daemon: ServiceDaemon,
    info: Option<ServiceInfo>,
    opt: SdResolverOptions<T>,
}

impl<T> std::fmt::Debug for SdResolver<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SdResolver")
            .field("info", &self.info)
            .finish()
    }
}

impl<T> HandlerResolver for SdResolver<T>
where
    T: Fn(ServiceDiscoveryEvent) + Send + Sync + 'static,
{
    type Opt = SdResolverOptions<T>;
    type Op = ServiceDiscoveryRequest;
    type Err = ServiceDiscoveryErrors;

    fn new(opt: Self::Opt) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        let daemon = ServiceDaemon::new()?;

        let monitor = daemon.monitor()?;

        let _ = ASYNC_RUNTIME.spawn(async move {
            while let Ok(event) = monitor.recv() {
                tracing::debug!(event =?event, "daemon event");
            }
        });

        Ok(Self {
            daemon,
            info: None,
            opt,
        })
    }

    async fn resolve(
        &mut self,
        op: &ServiceDiscoveryRequest,
    ) -> Result<<Self::Op as Operation>::Output, Self::Err> {
        match op {
            ServiceDiscoveryRequest::EnableService(id) => {
                self.handle_enable_server(id)?;

                Ok(ServiceDiscoveryEvent::None)
            }
            ServiceDiscoveryRequest::DisableService => {
                self.handle_disable_server().await?;

                Ok(ServiceDiscoveryEvent::None)
            }
            ServiceDiscoveryRequest::FindByHostname(hostname) => {
                let addresses = self.handle_search(hostname).await?;
                let service = LocalService {
                    hostname: hostname.to_owned(),
                    addresses: addresses.into_iter().map(ScopedIp::from).collect(),
                    fullname: String::new(),
                };

                Ok(ServiceDiscoveryEvent::AppearedService(service))
            }
            ServiceDiscoveryRequest::BeginBrowseForServices => {
                self.begin_browse().await?;

                Ok(ServiceDiscoveryEvent::None)
            }
            ServiceDiscoveryRequest::StopBrowseForServices => {
                self.stop_browse().await?;

                Ok(ServiceDiscoveryEvent::None)
            }
        }
    }
}

impl<T> SdResolver<T>
where
    T: Fn(ServiceDiscoveryEvent) + Send + Sync + 'static,
{
    pub fn handle_enable_server(&mut self, id: &uuid::Uuid) -> Result<(), ServiceDiscoveryErrors> {
        if self.info.is_some() {
            return Ok(());
        }

        let instance_name = data_encoding::BASE64.encode(id.as_bytes());

        let service_info = ServiceInfo::new(
            MDNS_SERVICE_TYPE,
            &instance_name,
            &HOSTNAME.to_mdns_name(),
            "",
            MDNS_SERVICE_PORT,
            Vec::new(),
        )
        .expect("service info for mdns daemon should be valid")
        .enable_addr_auto();

        self.daemon.register(service_info.clone())?;

        self.info = Some(service_info);

        Ok(())
    }

    pub async fn handle_disable_server(
        &mut self,
    ) -> Result<ServiceDiscoveryEvent, ServiceDiscoveryErrors> {
        let fullname = match self.info.as_ref() {
            Some(info) => info.get_fullname(),
            None => return Ok(ServiceDiscoveryEvent::None),
        };

        let rx = self.daemon.unregister(fullname)?;

        match rx.recv_async().await {
            Ok(status) => {
                tracing::info!(status = ?status, "unregistered mdns_sd service");
            }
            Err(_) => panic!("tried to fetch status of un-registration on disconnected channel"),
        }

        Ok(ServiceDiscoveryEvent::None)
    }

    pub async fn begin_browse(&mut self) -> Result<(), ServiceDiscoveryErrors> {
        tokio::spawn({
            let rx = self.daemon.browse(MDNS_SERVICE_TYPE)?;
            let callback = Arc::clone(&self.opt.service_events_callback);

            async move {
                while let Ok(event) = rx.recv_async().await {
                    match event {
                        mdns_sd::ServiceEvent::ServiceResolved(service) => {
                            if service.ty_domain == MDNS_SERVICE_TYPE {
                                (callback)(ServiceDiscoveryEvent::AppearedService(
                                    LocalService::from(*service),
                                ))
                            }
                        }
                        mdns_sd::ServiceEvent::ServiceRemoved(_, name) => {
                            (callback)(ServiceDiscoveryEvent::LostService(name));
                        }
                        _ => (),
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn stop_browse(&mut self) -> Result<(), ServiceDiscoveryErrors> {
        self.daemon.stop_browse(MDNS_SERVICE_TYPE)?;

        Ok(())
    }

    pub async fn handle_search(
        &mut self,
        hostname: &Hostname,
    ) -> Result<HashSet<mdns_sd::ScopedIp>, ServiceDiscoveryErrors> {
        let rx = self.daemon.resolve_hostname(
            &hostname.to_mdns_name(),
            Some(OPERATION_TIMEOUT.as_millis() as u64),
        )?;

        let (tx, rx_names) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            while let Ok(event) = rx.recv_async().await {
                if let HostnameResolutionEvent::AddressesFound(_, ips) = event {
                    let _ = tx.send(ips);
                    return;
                }
            }
        });

        let ips = rx_names.await.unwrap_or_default();

        Ok(ips)
    }
}
