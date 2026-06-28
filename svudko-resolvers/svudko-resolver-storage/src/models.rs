use sqlx::prelude::FromRow;
use svudko_common::hostname::Hostname;

#[derive(Debug, Clone)]
pub struct PairedDevice {
    pub hostname: Hostname,
    pub identifier: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct PairedDeviceRaw {
    pub hostname: String,
    pub identifier: String,
}

impl From<PairedDeviceRaw> for PairedDevice {
    fn from(
        PairedDeviceRaw {
            hostname,
            identifier,
        }: PairedDeviceRaw,
    ) -> Self {
        Self {
            hostname: Hostname::new(hostname),
            identifier,
        }
    }
}
