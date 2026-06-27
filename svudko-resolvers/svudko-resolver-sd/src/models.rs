use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use svudko_common::hostname::Hostname;

#[derive(Debug, Clone)]
pub struct LocalService {
    pub hostname: Hostname,
    pub fullname: String,

    /// Addresses of the service. IPv4 or IPv6 addresses.
    pub addresses: HashSet<ScopedIp>,
}

impl From<mdns_sd::ResolvedService> for LocalService {
    fn from(
        mdns_sd::ResolvedService {
            host,
            addresses,
            fullname,
            ..
        }: mdns_sd::ResolvedService,
    ) -> Self {
        Self {
            hostname: Hostname::new(host),
            addresses: addresses.into_iter().map(ScopedIp::from).collect(),
            fullname,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ScopedIp {
    V4(ScopedIpV4),
    V6(ScopedIpV6),
}

impl From<mdns_sd::ScopedIp> for ScopedIp {
    fn from(value: mdns_sd::ScopedIp) -> Self {
        match value {
            mdns_sd::ScopedIp::V4(addr) => Self::V4(ScopedIpV4 {
                addr: *addr.addr(),
                interface_ids: addr
                    .interface_ids()
                    .iter()
                    .cloned()
                    .map(InterfaceId::from)
                    .collect(),
            }),
            mdns_sd::ScopedIp::V6(addr) => Self::V6(ScopedIpV6 {
                addr: *addr.addr(),
                scope_id: addr.scope_id().clone().into(),
            }),
            _ => unreachable!("no other variants exists at current moment"),
        }
    }
}

impl ScopedIp {
    #[must_use]
    pub const fn is_loopback(&self) -> bool {
        match self {
            ScopedIp::V4(v4) => v4.addr.is_loopback(),
            ScopedIp::V6(v6) => v6.addr.is_loopback(),
        }
    }

    #[must_use]
    pub const fn to_ip_addr(&self) -> IpAddr {
        match self {
            ScopedIp::V4(v4) => IpAddr::V4(v4.addr),
            ScopedIp::V6(v6) => IpAddr::V6(v6.addr),
        }
    }

    #[must_use]
    pub const fn is_ipv4(&self) -> bool {
        matches!(self, ScopedIp::V4(_))
    }

    #[must_use]
    pub const fn is_ipv6(&self) -> bool {
        matches!(self, ScopedIp::V6(_))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ScopedIpV6 {
    pub addr: Ipv6Addr,
    pub scope_id: InterfaceId,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ScopedIpV4 {
    pub addr: Ipv4Addr,
    /// The interfaces this address was discovered on.
    pub interface_ids: Vec<InterfaceId>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Default)]
pub struct InterfaceId {
    /// Interface name, e.g. "en0", "wlan0", etc.
    pub name: String,

    /// Interface index assigned by the OS, e.g. 1, 2, etc.
    pub index: u32,
}

impl From<mdns_sd::InterfaceId> for InterfaceId {
    fn from(mdns_sd::InterfaceId { name, index }: mdns_sd::InterfaceId) -> Self {
        Self { name, index }
    }
}
