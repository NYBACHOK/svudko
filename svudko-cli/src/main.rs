use std::{
    io::BufRead,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    path::PathBuf,
    sync::{Arc, LazyLock, mpsc::Sender},
};

use anyhow::Context;
use svudko_common::{
    SERVER_PORT,
    quinn::{
        ClientConfig, Endpoint, ServerConfig, VarInt,
        crypto::rustls::QuicClientConfig,
        rustls::{
            self,
            pki_types::{CertificateDer, PrivatePkcs8KeyDer, pem::PemObject},
        },
    },
};
use svudko_common::{dummy_verification::SkipServerVerification, identity::load_or_generate_cert};
use svudko_core::{ApplicationCore, CruxShell, Event, resolvers::dns_sd::LocalDnsSdRequest};
use tokio::io::AsyncReadExt;

const DEFAULT_SERVER_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), SERVER_PORT);

/// Debug cli to test different app functions without implementation of real app behavior
#[derive(clap::Parser)]
#[non_exhaustive]
struct Args {
    #[command(subcommand)]
    subcommand: Mode,

    #[arg(long, global = true, required = false, default_value_os_t = APP_DATA_DIR.join("certificate.pem"))]
    cert_file: PathBuf,

    #[arg(long, global = true, required = false, default_value_os_t = APP_DATA_DIR.join("private_key.pem"))]
    key_file: PathBuf,
}

#[derive(clap::Subcommand)]
enum Mode {
    Client(ClientArgs),
    Server(ServerArgs),
    Dns(DnsArgs),
}

#[derive(clap::Args)]
struct ClientArgs {
    #[arg(short, long, required = false, default_value_t = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 0))]
    addr: SocketAddr,
    #[arg(required = false, default_value_t = DEFAULT_SERVER_ADDR)]
    connect_to: SocketAddr,
}

#[derive(clap::Args)]
struct ServerArgs {
    #[arg(short, long, required = false, default_value_t = DEFAULT_SERVER_ADDR)]
    addr: SocketAddr,
}

#[derive(clap::Args)]
struct DnsArgs {
    #[command(subcommand)]
    subcommand: DnsSubcommand,
}

#[derive(clap::Subcommand)]
enum DnsSubcommand {
    StartService,
    SearchForServices,
    SearchByHostname { name: String },
}

struct CliShell {
    tx: Sender<svudko_core::Effect>,
}

impl CruxShell for CliShell {
    fn process_effects(&self, effect: svudko_core::Effect) {
        let _ = self.tx.send(effect);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Args {
        cert_file,
        key_file,
        subcommand,
        ..
    } = <Args as clap::Parser>::parse();

    setup_logger();

    let cert = load_or_generate_cert(&cert_file, &key_file).await?;

    let (tx, rx) = std::sync::mpsc::channel();

    let shell = Arc::new(CliShell { tx });

    let core = ApplicationCore::new(Arc::clone(&shell));

    match subcommand {
        Mode::Client(ClientArgs { connect_to, addr }) => {
            let client_cfg = configure_client(cert_file)?;
            let mut endpoint = Endpoint::client(addr)?;
            endpoint.set_default_client_config(client_cfg);

            println!("Trying to connect");
            let connection = endpoint.connect(connect_to, "localhost")?.await?;

            let mut stream = connection.open_uni().await?;

            println!("Connected! RTT: {}ms", connection.rtt().as_millis());

            loop {
                let in_std = std::io::stdin();

                let mut input_lock = in_std.lock();

                let mut buffer = String::new();

                input_lock.read_line(&mut buffer)?;

                stream.write(buffer.as_bytes()).await?;

                stream.finish()?;

                connection.closed().await;

                break;
            }
        }
        Mode::Server(ServerArgs { addr }) => {
            let server_config = configure_server(&cert, &key_file)?;
            let endpoint = Endpoint::server(server_config, addr)?;

            println!(
                "started accepting connections at {}",
                endpoint.local_addr()?
            );

            loop {
                if let Some(connection) = endpoint.accept().await {
                    match connection.await {
                        Ok(connection) => {
                            println!("Received connection");

                            let mut stream = connection.accept_uni().await?;

                            let mut buffer = String::new();

                            stream.read_to_string(&mut buffer).await?;

                            println!("{}", buffer);

                            connection.close(VarInt::from_u32(0), "finished".as_bytes());
                        }
                        Err(e) => eprintln!("{e}"),
                    };
                } else {
                    break;
                }
            }
        }
        Mode::Dns(DnsArgs { subcommand }) => match subcommand {
            DnsSubcommand::StartService => {
                core.inner()
                    .update(Event::Dns(LocalDnsSdRequest::EnableService));
            }
            DnsSubcommand::SearchForServices => core
                .inner()
                .update(Event::Dns(LocalDnsSdRequest::BrowseForService)),
            DnsSubcommand::SearchByHostname { name } => core
                .inner()
                .update(Event::Dns(LocalDnsSdRequest::FindByHostname(name))),
        },
    };

    while let Ok(_effect) = rx.recv() {}

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
        .dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    let client_config = svudko_common::quinn::ClientConfig::new(Arc::new(
        QuicClientConfig::try_from(client_crypto)?,
    ));

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

pub fn setup_logger() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    #[allow(unused_mut)]
    let mut registry = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(
                    match cfg!(debug_assertions) {
                        true => tracing::Level::DEBUG,
                        false => tracing::Level::INFO,
                    }
                    .into(),
                )
                .from_env()
                .expect("default level is set")
                .add_directive("hyper_util=warn".parse().unwrap())
                .add_directive("reqwest=warn".parse().unwrap()),
        );

    registry.init();
}
