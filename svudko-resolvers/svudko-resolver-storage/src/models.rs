use sqlx::prelude::FromRow;
use svudko_common::hostname::Hostname;

#[derive(Debug, Clone)]
pub struct TrustedHost {
    pub hostname: Hostname,
    pub signature: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct TrustedHostRaw {
    pub hostname: String,
    pub signature: String,
}

impl From<TrustedHostRaw> for TrustedHost {
    fn from(
        TrustedHostRaw {
            hostname,
            signature,
        }: TrustedHostRaw,
    ) -> Self {
        Self {
            hostname: Hostname::new(hostname),
            signature,
        }
    }
}
