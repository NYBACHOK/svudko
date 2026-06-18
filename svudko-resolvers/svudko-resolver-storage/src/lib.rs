pub mod models;
use std::path::PathBuf;

use sqlx::SqlitePool;
use svudko_common::{
    APP_DATA_DIR, ASYNC_RUNTIME,
    resolver::{HandlerResolver, Operation},
};

mod errors;
pub mod event;
pub mod request;
mod setup;

pub use errors::*;

use crate::{event::StorageEvent, models::TrustedHost, request::StorageRequest, setup::setup_db};

const APPLY_MIGRATIONS: bool = true;

pub fn database_path() -> PathBuf {
    const DB_FILE_NAME: &str = "svudko.db";

    APP_DATA_DIR.join(DB_FILE_NAME)
}

#[derive(Clone, Debug)]
pub struct StorageResolver {
    pool: SqlitePool,
}

impl HandlerResolver for StorageResolver {
    type Opt = ();
    type Op = StorageRequest;
    type Err = StorageErrors;

    fn new((): Self::Opt) -> Result<Self, Self::Err>
    where
        Self: Sized,
    {
        Ok(Self {
            pool: ASYNC_RUNTIME.block_on(setup_db())?,
        })
    }

    async fn resolve(
        &mut self,
        op: &Self::Op,
    ) -> Result<<Self::Op as Operation>::Output, Self::Err> {
        let event = match op {
            StorageRequest::Fetch => StorageEvent::Fetch(self.trusted_hosts().await?),
            StorageRequest::NewHost {
                host:
                    TrustedHost {
                        hostname,
                        signature,
                    },
                overwrite,
            } => {
                if *overwrite {
                    let host = self.trusted_host_overwrite(hostname, signature).await?;
                    StorageEvent::HostAdded(host)
                } else {
                    match self.trusted_host_add(hostname, signature).await? {
                        Some(host) => StorageEvent::HostAdded(host),
                        None => StorageEvent::HostAlreadyExists(hostname.to_owned()),
                    }
                }
            }
        };

        Ok(event)
    }
}

impl StorageResolver {
    pub async fn trusted_hosts(&self) -> Result<Vec<TrustedHost>, StorageErrors> {
        sqlx::query_as("SELECT * FROM trusted_hosts;")
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn trusted_host_add(
        &self,
        hostname: &str,
        signature: &str,
    ) -> Result<Option<TrustedHost>, StorageErrors> {
        sqlx::query_as::<_, TrustedHost>(
            " INSERT OR IGNORE INTO trusted_hosts (hostname, signature) VALUES ($1, $2) RETURNING *;",
        )
        .bind(hostname)
        .bind(signature)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn trusted_host_overwrite(
        &self,
        hostname: &str,
        signature: &str,
    ) -> Result<TrustedHost, StorageErrors> {
        sqlx::query_as::<_, TrustedHost>(
            " INSERT INTO trusted_hosts (hostname, signature) VALUES ($1, $2) ON CONFLICT (hostname) DO UPDATE SET signature = excluded.signature RETURNING *;",
        )
        .bind(hostname)
        .bind(signature)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }
}
