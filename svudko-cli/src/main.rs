use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    path::PathBuf,
    sync::{Arc, LazyLock},
    time::Duration,
};

use anyhow::Context;
use quinn::{
    ClientConfig, Endpoint, ServerConfig,
    crypto::rustls::QuicClientConfig,
    rustls::{
        self,
        pki_types::{CertificateDer, PrivatePkcs8KeyDer, pem::PemObject},
    },
};

#[derive(clap::Parser)]
#[non_exhaustive]
struct Args {
    #[arg(short, long, required = false, default_value_t = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 0))]
    addr: SocketAddr,

    #[arg(short, long, required = false)]
    connect_to: Option<SocketAddr>,

    #[arg(long, required = false, default_value_os_t = APP_DATA_DIR.join("certificate.pem"))]
    cert_file: PathBuf,

    #[arg(long, required = false, default_value_os_t = APP_DATA_DIR.join("private_key.pem"))]
    key_file: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Args {
        addr,
        connect_to,
        cert_file,
        key_file,
        ..
    } = <Args as clap::Parser>::parse();

    let cert = load_or_generate_cert(&cert_file, &key_file)?;

    let server_config = configure_server(&cert, &key_file)?;
    let client_cfg = configure_client(cert_file)?;
    let mut endpoint = Endpoint::server(server_config, addr)?;

    endpoint.set_default_client_config(client_cfg);

    tokio::spawn({
        let endpoint = endpoint.clone();
        println!("{}", endpoint.local_addr().unwrap());

        async move {
            println!("started accepting connections");

            loop {
                if let Some(connection) = endpoint.accept().await {
                    match connection.await {
                        Ok(_connection) => {
                            println!("Received connection")
                        }
                        Err(e) => eprintln!("{e}"),
                    };
                } else {
                    return;
                }
            }
        }
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    if let Some(connect_to) = connect_to {
        println!("Trying to connect");
        let connection = endpoint.connect(connect_to, "localhost")?.await?;

        println!("Connected! RTT: {}ms", connection.rtt().as_millis());
    }

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

fn configure_server(
    cert_der: &CertificateDer<'static>,
    key_file: &PathBuf,
) -> Result<ServerConfig, anyhow::Error> {
    let priv_key = PrivatePkcs8KeyDer::from_pem_file(key_file)?;

    let mut server_config =
        ServerConfig::with_single_cert(vec![cert_der.clone()], priv_key.into())?;
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(1_u8.into());

    Ok(server_config)
}

fn configure_client(cert_file: PathBuf) -> Result<ClientConfig, anyhow::Error> {
    let mut certs = rustls::RootCertStore::empty();
    certs.add(CertificateDer::from_pem_file(cert_file).context("failed load cert for client")?)?;

    let client_crypto = rustls::ClientConfig::builder()
        .with_root_certificates(certs)
        .with_no_client_auth();

    let client_config =
        quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(client_crypto)?));

    Ok(client_config)
}

static APP_DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    // TODO: for tests I need use tmp_dir
    const BUNDLE_ID: &str = "svukdo";

    data_dir().join(BUNDLE_ID)
});

#[inline]
fn data_dir() -> PathBuf {
    #[cfg(target_os = "android")]
    {
        PathBuf::from("/data/data")
    }
    #[cfg(not(target_os = "android"))]
    {
        dirs::data_dir().unwrap_or_else(|| {
            let dir = std::env::current_dir().unwrap_or_default();

            tracing::error!(data_dir = %dir.display(), "failed to get data dir will ");

            dir
        })
    }
}

fn load_or_generate_cert(
    cert_file: &PathBuf,
    key_file: &PathBuf,
) -> anyhow::Result<CertificateDer<'static>> {
    if cert_file.exists() && key_file.exists() {
        // Load from disk
        let cert_pem = std::fs::read_to_string(cert_file)?;
        return Ok(CertificateDer::from_pem_slice(cert_pem.as_bytes())?.into_owned());
    }

    // Generate if not exists
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    std::fs::write(cert_file, cert.cert.pem())?;
    std::fs::write(key_file, cert.signing_key.serialize_pem())?;
    Ok(cert.cert.into())
}
