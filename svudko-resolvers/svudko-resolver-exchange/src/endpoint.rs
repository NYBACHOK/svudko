use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, RwLock},
};

use quinn::{
    ClientConfig, Endpoint, ServerConfig,
    crypto::rustls::{QuicClientConfig, QuicServerConfig},
    rustls::{
        self,
        pki_types::{CertificateDer, PrivatePkcs8KeyDer, pem::PemObject},
    },
};
use rcgen::{Issuer, SanType};
use svudko_common::{APP_DATA_DIR, CERT_CA_KEY_PEM};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    ExchangeErrors,
    models::UnknownSignature,
    verification::{client::WhiteListClientVerifier, server::DisabledServerVerifier},
};

static ROOT_CERT: LazyLock<Issuer<'static, rcgen::KeyPair>> = LazyLock::new(|| {
    Issuer::from_ca_cert_pem(
        svudko_common::CERT_CA_PEM,
        rcgen::KeyPair::from_pem(CERT_CA_KEY_PEM).expect("always valid"),
    )
    .expect("always valid")
});

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
    certs.add(CertificateDer::from_pem_slice(
        svudko_common::CERT_CA_PEM.as_bytes(),
    )?)?;

    let client_crypto = rustls::ClientConfig::builder()
        .with_root_certificates(certs)
        .with_no_client_auth();

    let client_config = ClientConfig::new(Arc::new(QuicClientConfig::try_from(client_crypto)?));

    Ok(client_config)
}

pub async fn load_or_generate_cert(
    cert_file: &Path,
    key_file: &Path,
    names: Vec<SanType>,
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
    params.subject_alt_names = names;

    let key_pair = rcgen::KeyPair::generate()?;

    let cert = params.signed_by(&key_pair, &*ROOT_CERT)?;

    tokio::fs::write(cert_file, cert.pem()).await?;
    tokio::fs::write(key_file, key_pair.serialize_pem()).await?;

    Ok(cert.into())
}

pub async fn endpoint(
    addr: SocketAddr,
    names: Vec<SanType>,
    trusted_hosts: Arc<RwLock<HashMap<String, String>>>,
    tx: UnboundedSender<UnknownSignature>,
) -> Result<Endpoint, ExchangeErrors> {
    if !APP_DATA_DIR.exists() {
        tokio::fs::create_dir_all(&*APP_DATA_DIR).await?;
    }

    let cert_file = APP_DATA_DIR.join("certificate.pem");
    let key_file = APP_DATA_DIR.join("private_key.pem");
    let cert = load_or_generate_cert(&cert_file, &key_file, names).await?;

    let client_cfg = configure_client(cert_file)?;
    let server_cfg = configure_server(&cert, &key_file, trusted_hosts, tx)?;

    let mut endpoint = Endpoint::server(server_cfg, addr)?;
    endpoint.set_default_client_config(client_cfg);

    Ok(endpoint)
}
