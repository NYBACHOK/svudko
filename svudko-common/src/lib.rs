use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    path::PathBuf,
    sync::{LazyLock, OnceLock},
};

pub mod resolver;

pub static ASYNC_RUNTIME: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().expect("failed to init runtime"));

pub const DEFAULT_SERVER_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), SERVER_PORT);

#[cfg(debug_assertions)]
pub const CERT_CA_PEM: &str = include_str!("../../build/debug-keys/rootCA.pem");
#[cfg(not(debug_assertions))]
pub const CERT_CA_PEM: &str = include_str!("../../build/release-keys/rootCA.pem");

#[cfg(debug_assertions)]
pub const CERT_CA_KEY_PEM: &str = include_str!("../../build/debug-keys/rootCA.key");
#[cfg(not(debug_assertions))]
pub const CERT_CA_KEY_PEM: &str = include_str!("../../build/release-keys/rootCA.key");

pub const SERVER_PORT: u16 = 4443;

pub static APP_DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
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

pub fn hostname() -> &'static str {
    static HOST_NAME: OnceLock<String> = OnceLock::new();

    HOST_NAME.get_or_init(|| {
        gethostname::gethostname()
            .to_string_lossy()
            .replace(char::REPLACEMENT_CHARACTER, "")
    })
}
