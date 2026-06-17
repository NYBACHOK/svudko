use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use crux_core::capability::Operation;
use mdns_sd::{HostnameResolutionEvent, ResolvedService, ScopedIp, ServiceDaemon, ServiceInfo};
use svudko_common::resolver::HandlerResolver;

use crate::{event::LocalDnsSdEvent, request::LocalDnsSdRequest};

pub mod event;
pub mod request;

const OPERATION_TIMEOUT: Duration = Duration::from_secs(20);

pub const MDNS_SERVICE_PORT: u16 = 15571;
pub const MDNS_SERVICE_TYPE: &str = "_svudko-app._udp.local.";

#[derive(Clone, Debug, thiserror::Error)]
pub enum DnsSdErrors {
    #[error(transparent)]
    Mdns(#[from] mdns_sd::Error),
}

#[derive(Clone)]
pub struct SdResolver {
    daemon: ServiceDaemon,
    info: Option<ServiceInfo>,
}

impl std::fmt::Debug for SdResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SdResolver")
            .field("info", &self.info)
            .finish()
    }
}

impl HandlerResolver for SdResolver {
    type Opt = ();
    type Op = LocalDnsSdRequest;
    type Err = DnsSdErrors;

    fn new((): Self::Opt) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        let daemon = ServiceDaemon::new()?;

        let monitor = daemon.monitor()?;

        let _ = tokio::spawn(async move {
            while let Ok(event) = monitor.recv() {
                tracing::debug!(event =?event, "daemon event");
            }
        });

        Ok(Self { daemon, info: None })
    }

    async fn resolve(
        &mut self,
        op: &LocalDnsSdRequest,
    ) -> Result<<Self::Op as Operation>::Output, Self::Err> {
        match op {
            LocalDnsSdRequest::EnableService => {
                self.handle_enable_server()?;

                Ok(LocalDnsSdEvent::Enabled)
            }
            LocalDnsSdRequest::DisableService => {
                self.handle_disable_server().await?;

                Ok(LocalDnsSdEvent::Disabled)
            }
            LocalDnsSdRequest::BrowseForServices => {
                let services = self.handle_browse().await?;

                Ok(LocalDnsSdEvent::FoundServices(services))
            }
            LocalDnsSdRequest::FindByHostname(hostname) => {
                let ips = self.handle_search(hostname).await?;

                Ok(LocalDnsSdEvent::FoundIps(ips))
            }
        }
    }
}

impl SdResolver {
    pub fn handle_enable_server(&mut self) -> Result<(), DnsSdErrors> {
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

    pub async fn handle_disable_server(&mut self) -> Result<LocalDnsSdEvent, DnsSdErrors> {
        let fullname = match self.info.as_ref() {
            Some(info) => info.get_fullname(),
            None => return Ok(LocalDnsSdEvent::Disabled),
        };

        let rx = self.daemon.unregister(fullname)?;

        match rx.recv_async().await {
            Ok(status) => {
                tracing::info!(status = ?status, "unregistered mdns_sd service");
            }
            Err(_) => panic!("tried to fetch status of un-registration on disconnected channel"),
        }

        Ok(LocalDnsSdEvent::Disabled)
    }

    pub async fn handle_browse(
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

    pub async fn handle_search(
        &mut self,
        hostname: &str,
    ) -> Result<HashSet<ScopedIp>, DnsSdErrors> {
        let rx = self
            .daemon
            .resolve_hostname(hostname, Some(OPERATION_TIMEOUT.as_millis() as u64))?;

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
