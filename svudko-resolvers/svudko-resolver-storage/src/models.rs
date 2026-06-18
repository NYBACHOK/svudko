use sqlx::prelude::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct TrustedHost {
    pub hostname: String,
    pub signature: String,
}
