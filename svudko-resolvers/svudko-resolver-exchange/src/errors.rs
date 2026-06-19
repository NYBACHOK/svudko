#[derive(Debug, thiserror::Error)]
pub enum ExchangeErrors {
    #[error("failed to start resolver. Reason: {0}")]
    Initialize(#[from] ExchangeInitializeErrors),
    #[error(transparent)]
    Tmp(#[from] anyhow::Error),
    #[error(transparent)]
    Connection(#[from] quinn::ConnectionError),
    #[error(transparent)]
    Connect(#[from] quinn::ConnectError),
}

#[derive(Debug, thiserror::Error)]
pub enum ExchangeInitializeErrors {
    #[error(transparent)]
    Connection(#[from] quinn::ConnectionError),
    #[error(transparent)]
    Connect(#[from] quinn::ConnectError),
    #[error("{0}")]
    Io(String),
    #[error(transparent)]
    Tls(#[from] quinn::rustls::pki_types::pem::Error),
    #[error(transparent)]
    Rustls(#[from] quinn::rustls::Error),
    #[error(transparent)]
    NoInitialCipherSuite(#[from] quinn::crypto::rustls::NoInitialCipherSuite),
    #[error(transparent)]
    Rcgen(#[from] rcgen::Error),
}

impl From<std::io::Error> for ExchangeInitializeErrors {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}
