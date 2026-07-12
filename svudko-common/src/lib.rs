pub mod hostname;
use std::{
    net::{IpAddr, Ipv6Addr, SocketAddr},
    path::PathBuf,
    sync::LazyLock,
};

pub mod resolver;

pub const POISONED_LOCK_MSG: &str = "poisoned lock";

pub static ASYNC_RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    #[cfg(debug_assertions)]
    {
        builder.thread_stack_size(8 * 1024 * 1024);
    }

    builder
        .enable_all()
        .build()
        .expect("failed to init runtime")
});

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

    data_dir().join(if cfg!(debug_assertions) {
        "svukdo/debug"
    } else {
        BUNDLE_ID
    })
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
