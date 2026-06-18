use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use quinn::{
    ClientConfig, Endpoint, ServerConfig,
    crypto::rustls::{QuicClientConfig, QuicServerConfig},
    rustls::{
        self,
        pki_types::{CertificateDer, PrivatePkcs8KeyDer, pem::PemObject},
    },
};
use svudko_common::APP_DATA_DIR;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    ExchangeErrors,
    models::UnknownSignature,
    verification::{client::WhiteListClientVerifier, server::DisabledServerVerifier},
};

fn configure_server(
    cert_der: &CertificateDer<'static>,
    key_file: &PathBuf,
    trusted_hosts: Arc<RwLock<HashMap<String, String>>>,
    tx: UnboundedSender<UnknownSignature>,
) -> Result<ServerConfig, ExchangeErrors> {
    let priv_key = PrivatePkcs8KeyDer::from_pem_file(key_file)?;

    let tofu = WhiteListClientVerifier::new(trusted_hosts, tx);

    let tls_server = rustls::ServerConfig::builder()
        .with_client_cert_verifier(Arc::new(tofu))
        .with_single_cert(vec![cert_der.clone()], priv_key.into())?;

    let quic_crypto = QuicServerConfig::try_from(tls_server)?;
    let mut server_config = ServerConfig::with_crypto(Arc::new(quic_crypto));

    let transport_config =
        Arc::get_mut(&mut server_config.transport).expect("no other instances exists");
    transport_config.max_concurrent_uni_streams(1_u8.into());

    Ok(server_config)
}

fn configure_client(cert_file: PathBuf) -> Result<ClientConfig, ExchangeErrors> {
    let mut certs = rustls::RootCertStore::empty();
    certs.add(CertificateDer::from_pem_file(cert_file)?)?;

    let client_crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(DisabledServerVerifier::new())
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

    let mut params = rcgen::CertificateParams::default();
    params.distinguished_name = rcgen::DistinguishedName::new();
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, svudko_common::hostname());

    let key_pair = rcgen::KeyPair::generate()?;

    let cert = params.self_signed(&key_pair)?;

    tokio::fs::write(cert_file, cert.pem()).await?;
    tokio::fs::write(key_file, key_pair.serialize_pem()).await?;

    Ok(cert.into())
}

pub async fn endpoint(
    addr: SocketAddr,
    trusted_hosts: Arc<RwLock<HashMap<String, String>>>,
    tx: UnboundedSender<UnknownSignature>,
) -> Result<Endpoint, ExchangeErrors> {
    if !APP_DATA_DIR.exists() {
        tokio::fs::create_dir_all(&*APP_DATA_DIR).await?;
    }

    let cert_file = APP_DATA_DIR.join("certificate.pem");
    let key_file = APP_DATA_DIR.join("private_key.pem");
    let cert = load_or_generate_cert(&cert_file, &key_file).await?;

    let client_cfg = configure_client(cert_file)?;
    let server_cfg = configure_server(&cert, &key_file, trusted_hosts, tx)?;

    let mut endpoint = Endpoint::server(server_cfg, addr)?;
    endpoint.set_default_client_config(client_cfg);

    Ok(endpoint)
}
