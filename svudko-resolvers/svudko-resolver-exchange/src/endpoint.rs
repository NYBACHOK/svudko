use std::{
    collections::HashSet,
    net::SocketAddr,
    path::Path,
    sync::{Arc, LazyLock, RwLock},
};

use quinn::{
    ClientConfig, Endpoint, ServerConfig,
    crypto::rustls::{QuicClientConfig, QuicServerConfig},
    rustls::{
        self,
        pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
    },
};
use rcgen::{Issuer, SanType};
use svudko_common::{APP_DATA_DIR, CERT_CA_KEY_PEM, hostname::HOSTNAME};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    errors::ExchangeInitializeErrors, models::UnknownSignature,
    verification::client::WhiteListClientVerifier,
};

static ROOT_CERT: LazyLock<CertificateDer<'static>> = LazyLock::new(|| {
    CertificateDer::from_pem_slice(svudko_common::CERT_CA_PEM.as_bytes()).expect("alway valid")
});

fn configure_server(
    cert_der: CertificateDer<'static>,
    key_der: PrivateKeyDer<'static>,
    trusted_signatures: Arc<RwLock<HashSet<String>>>,
    tx: UnboundedSender<UnknownSignature>,
) -> Result<ServerConfig, ExchangeInitializeErrors> {
    let tofu = WhiteListClientVerifier::new(trusted_signatures, tx);

    let tls_server = rustls::ServerConfig::builder()
        .with_client_cert_verifier(Arc::new(tofu))
        .with_single_cert(vec![cert_der], key_der)?;

    let quic_crypto = QuicServerConfig::try_from(tls_server)?;
    let mut server_config = ServerConfig::with_crypto(Arc::new(quic_crypto));

    let transport_config =
        Arc::get_mut(&mut server_config.transport).expect("no other instances exists");
    transport_config.max_concurrent_uni_streams(1_u8.into());

    Ok(server_config)
}

fn configure_client(
    cert_der: CertificateDer<'static>,
    key_der: PrivateKeyDer<'static>,
) -> Result<ClientConfig, ExchangeInitializeErrors> {
    let mut certs = rustls::RootCertStore::empty();

    certs.add(ROOT_CERT.clone())?;

    let client_crypto = rustls::ClientConfig::builder()
        .with_root_certificates(certs)
        .with_client_auth_cert(vec![cert_der], key_der)?;

    Ok(ClientConfig::new(Arc::new(QuicClientConfig::try_from(
        client_crypto,
    )?)))
}

async fn load_or_generate_cert(
    cert_file: &Path,
    key_file: &Path,
    names: Vec<SanType>,
) -> Result<(CertificateDer<'static>, rcgen::KeyPair), ExchangeInitializeErrors> {
    if cert_file.exists() && key_file.exists() {
        let key_pem = std::fs::read_to_string(key_file)?;
        return Ok((
            CertificateDer::from_pem_file(cert_file)?,
            rcgen::KeyPair::from_pem(&key_pem)?,
        ));
    }

    let mut params = rcgen::CertificateParams::default();
    params.distinguished_name = rcgen::DistinguishedName::new();
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, HOSTNAME.clone());
    params.subject_alt_names = names;

    let key_pair = rcgen::KeyPair::generate()?;

    let cert = params.signed_by(
        &key_pair,
        &Issuer::from_ca_cert_pem(
            svudko_common::CERT_CA_PEM,
            rcgen::KeyPair::from_pem(CERT_CA_KEY_PEM).expect("always valid"),
        )
        .expect("always valid"),
    )?;

    tokio::fs::write(cert_file, cert.pem()).await?;
    tokio::fs::write(key_file, key_pair.serialize_pem()).await?;

    Ok((cert.into(), key_pair))
}

pub async fn endpoint(
    addr: SocketAddr,
    names: Vec<SanType>,
    trusted_signatures: Arc<RwLock<HashSet<String>>>,
    tx: UnboundedSender<UnknownSignature>,
) -> Result<Endpoint, ExchangeInitializeErrors> {
    if !APP_DATA_DIR.exists() {
        tokio::fs::create_dir_all(&*APP_DATA_DIR).await?;
    }

    let cert_file = APP_DATA_DIR.join("certificate.pem");
    let key_file = APP_DATA_DIR.join("private_key.pem");
    let (cert, keypair) = load_or_generate_cert(&cert_file, &key_file, names).await?;

    let priv_key = PrivateKeyDer::from(keypair);

    let client_cfg = configure_client(cert.clone(), priv_key.clone_key())?;
    let server_cfg = configure_server(cert, priv_key, trusted_signatures, tx)?;

    let mut endpoint = Endpoint::server(server_cfg, addr)?;
    endpoint.set_default_client_config(client_cfg);

    Ok(endpoint)
}
