use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use quinn::{
    ClientConfig, Endpoint, ServerConfig,
    crypto::rustls::QuicClientConfig,
    rustls::{
        self,
        pki_types::{CertificateDer, PrivatePkcs8KeyDer, pem::PemObject},
    },
};
use svudko_common::{APP_DATA_DIR, dummy_verification::SkipServerVerification};

use crate::ExchangeErrors;

pub const DOMAIN: &str = "app.sync.svudko";

fn configure_server(
    cert_der: &CertificateDer<'static>,
    key_file: &PathBuf,
) -> Result<ServerConfig, ExchangeErrors> {
    let priv_key = PrivatePkcs8KeyDer::from_pem_file(key_file)?;

    let mut server_config =
        ServerConfig::with_single_cert(vec![cert_der.clone()], priv_key.into())?;
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(1_u8.into());

    Ok(server_config)
}

fn configure_client(cert_file: PathBuf) -> Result<ClientConfig, ExchangeErrors> {
    let mut certs = rustls::RootCertStore::empty();
    certs.add(CertificateDer::from_pem_file(cert_file)?)?;

    let client_crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    let client_config = ClientConfig::new(Arc::new(QuicClientConfig::try_from(client_crypto)?));

    Ok(client_config)
}

pub async fn load_or_generate_cert(
    cert_file: &Path,
    key_file: &Path,
) -> Result<CertificateDer<'static>, ExchangeErrors> {
    if cert_file.exists() && key_file.exists() {
        let cert_pem = std::fs::read_to_string(cert_file)?;
        return Ok(CertificateDer::from_pem_slice(cert_pem.as_bytes())?.into_owned());
    }

    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_owned(), DOMAIN.to_owned()])?;

    let (write_cert, write_signing) = tokio::join!(
        tokio::fs::write(cert_file, cert.cert.pem()),
        tokio::fs::write(key_file, cert.signing_key.serialize_pem())
    );

    write_cert?;
    write_signing?;

    Ok(cert.cert.into())
}

pub async fn endpoint(addr: SocketAddr) -> Result<Endpoint, ExchangeErrors> {
    if !APP_DATA_DIR.exists() {
        tokio::fs::create_dir_all(&*APP_DATA_DIR).await?;
    }

    let cert_file = APP_DATA_DIR.join("certificate.pem");
    let key_file = APP_DATA_DIR.join("private_key.pem");
    let cert = load_or_generate_cert(&cert_file, &key_file).await?;

    let client_cfg = configure_client(cert_file)?;
    let server_cfg = configure_server(&cert, &key_file)?;

    let mut endpoint = Endpoint::server(server_cfg, addr)?;
    endpoint.set_default_client_config(client_cfg);

    Ok(endpoint)
}
