#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("Failed to connect to DB. Reason: {0}")]
    ConnectionFail(#[from] sqlx::Error),
    #[error("Failed to apply migration. Reason: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[cfg(debug_assertions)]
    #[error("Failed to delete db in debug. Reason: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum StorageErrors {
    #[error("Failed to initialize db. Reason: {0}")]
    Setup(#[from] ConnectError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
