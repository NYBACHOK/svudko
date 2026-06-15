use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use crux_core::capability::Operation;
use facet::Facet;
use mdns_sd::{
    DaemonEvent, HostnameResolutionEvent, Receiver, ResolvedService, ScopedIp, ServiceDaemon,
    ServiceInfo,
};
use serde::{Deserialize, Serialize};
use svudko_common::{MDNS_SERVICE_PORT, MDNS_SERVICE_TYPE};

use super::*;

const OPERATION_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Facet, Serialize, Deserialize, Clone, Debug, thiserror::Error)]
#[repr(C)]
pub enum DnsSdErrors {
    #[serde(skip)]
    #[error(transparent)]
    Mdns(
        #[facet(opaque)]
        #[from]
        mdns_sd::Error,
    ),
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum LocalDnsSdRequest {
    EnableService,
    DisableService,

    BrowseForService,
    FindByHostname(String),
}

#[derive(Facet, Serialize, Deserialize, Clone, Debug)]
#[repr(C)]
pub enum LocalDnsSdResponse {
    Enabled,
    Disabled,
    #[facet(skip)]
    #[serde(skip)]
    FoundServices(#[facet(opaque)] HashMap<String, Box<ResolvedService>>),
    #[facet(skip)]
    #[serde(skip)]
    FoundIps(#[facet(opaque)] HashSet<ScopedIp>),
}

impl Operation for LocalDnsSdRequest {
    type Output = Result<LocalDnsSdResponse, DnsSdErrors>;
}

#[derive(Clone)]
pub struct Resolver {
    daemon: ServiceDaemon,
    _monitor: Receiver<DaemonEvent>,
    info: Option<ServiceInfo>,
}

impl Resolver {
    pub fn new() -> Result<Self, mdns_sd::Error> {
        let daemon = ServiceDaemon::new()?;

        let monitor = daemon.monitor()?;

        Ok(Self {
            daemon,
            _monitor: monitor,
            info: None,
        })
    }
}

impl HandlerResolver<LocalDnsSdRequest, <LocalDnsSdRequest as Operation>::Output> for Resolver {
    async fn resolve(
        &mut self,
        op: &LocalDnsSdRequest,
    ) -> <LocalDnsSdRequest as Operation>::Output {
        match op {
            LocalDnsSdRequest::EnableService => {
                self.handle_enable_server()?;

                Ok(LocalDnsSdResponse::Enabled)
            }
            LocalDnsSdRequest::DisableService => {
                self.handle_disable_server().await?;

                Ok(LocalDnsSdResponse::Disabled)
            }
            LocalDnsSdRequest::BrowseForService => {
                let services = self.handle_browse().await?;

                Ok(LocalDnsSdResponse::FoundServices(services))
            }
            LocalDnsSdRequest::FindByHostname(hostname) => {
                let ips = self.handle_search(&hostname).await?;

                Ok(LocalDnsSdResponse::FoundIps(ips))
            }
        }
    }
}

impl Resolver {
    fn handle_enable_server(&mut self) -> Result<(), DnsSdErrors> {
        if self.info.is_some() {
            return Ok(());
        }

        let instance_name = data_encoding::BASE64.encode(uuid::Uuid::new_v4().as_bytes());

        let host_name = format!(
            "_{}.local.",
            gethostname::gethostname()
                .to_string_lossy()
                .replace(char::REPLACEMENT_CHARACTER, ""),
        );

        let service_info = ServiceInfo::new(
            MDNS_SERVICE_TYPE,
            &instance_name,
            &host_name,
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

    async fn handle_disable_server(&mut self) -> Result<LocalDnsSdResponse, DnsSdErrors> {
        let fullname = match self.info.as_ref() {
            Some(info) => info.get_fullname(),
            None => return Ok(LocalDnsSdResponse::Disabled),
        };

        let rx = self.daemon.unregister(fullname)?;

        match rx.recv_async().await {
            Ok(status) => {
                tracing::info!(status = ?status, "unregistered mdns_sd service");
            }
            Err(_) => panic!("tried to fetch status of un-registration on disconnected channel"),
        };

        Ok(LocalDnsSdResponse::Disabled)
    }

    async fn handle_browse(
        &mut self,
    ) -> Result<HashMap<String, Box<ResolvedService>>, DnsSdErrors> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        tokio::spawn({
            let rx = self.daemon.browse(MDNS_SERVICE_TYPE)?;

            async move {
                let mut services = HashMap::new();

                while let Ok(event) = rx.recv_async().await {
                    match event {
                        mdns_sd::ServiceEvent::ServiceResolved(service) => {
                            services.insert(service.fullname.clone(), service);
                        }
                        mdns_sd::ServiceEvent::ServiceRemoved(_, name) => {
                            services.remove(&name);
                        }
                        _ => (),
                    }
                }

                let _ = tx.send(services);
            }
        });

        tokio::time::sleep(OPERATION_TIMEOUT).await;

        self.daemon.stop_browse(MDNS_SERVICE_TYPE)?;

        let services = rx.await.expect("always sends");

        Ok(services)
    }

    async fn handle_search(&mut self, hostname: &str) -> Result<HashSet<ScopedIp>, DnsSdErrors> {
        let rx = self
            .daemon
            .resolve_hostname(hostname, Some(OPERATION_TIMEOUT.as_millis() as u64))?;

        let (tx, rx_names) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            while let Ok(event) = rx.recv_async().await {
                match event {
                    HostnameResolutionEvent::AddressesFound(_, ips) => {
                        let _ = tx.send(ips);
                        return;
                    }
                    _ => (),
                }
            }
        });

        let ips = rx_names.await.unwrap_or_default();

        Ok(ips)
    }
}
