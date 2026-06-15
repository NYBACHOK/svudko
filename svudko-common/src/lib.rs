pub mod dummy_verification;
pub mod identity;

pub mod quinn {
    pub use quinn::*;
}

pub const SERVER_PORT: u16 = 4443;
pub const MDNS_SERVICE_PORT: u16 = 15571;
pub const MDNS_SERVICE_TYPE: &str = "_svudko-app._udp.local.";
