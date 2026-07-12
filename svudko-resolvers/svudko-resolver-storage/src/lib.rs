pub mod models;
use std::path::PathBuf;

use sqlx::SqlitePool;
use svudko_common::{
    APP_DATA_DIR, ASYNC_RUNTIME,
    hostname::Hostname,
    resolver::{HandlerResolver, Operation},
};

mod errors;
pub mod event;
pub mod request;
mod setup;

pub use errors::*;

use crate::{
    event::StorageEvent,
    models::{DeviceId, PairedDevice, PairedDeviceRaw},
    request::StorageRequest,
    setup::setup_db,
};

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
        let pool = ASYNC_RUNTIME.block_on(setup_db())?;

        Ok(Self { pool })
    }

    async fn resolve(
        &mut self,
        op: &Self::Op,
    ) -> Result<<Self::Op as Operation>::Output, Self::Err> {
        let event = match op {
            StorageRequest::Fetch => StorageEvent::Fetch(self.trusted_hosts().await?),
            StorageRequest::NewHost {
                hostname,
                identifier,
                overwrite,
            } => {
                if *overwrite {
                    let host = self.trusted_host_overwrite(hostname, identifier).await?;
                    StorageEvent::DeviceAdded(host)
                } else {
                    match self.trusted_host_add(hostname, identifier).await? {
                        Some(host) => StorageEvent::DeviceAdded(host),
                        None => StorageEvent::DeviceAlreadyExists(hostname.to_owned()),
                    }
                }
            }
            StorageRequest::ClientId => {
                let id = self.client_id().await?;

                StorageEvent::ClientId(id)
            }
        };

        Ok(event)
    }
}

impl StorageResolver {
    pub async fn trusted_hosts(&self) -> Result<Vec<PairedDevice>, StorageErrors> {
        sqlx::query_as::<_, PairedDeviceRaw>("SELECT * FROM paired_devices;")
            .fetch_all(&self.pool)
            .await
            .map(|this| this.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    pub async fn trusted_host_add(
        &self,
        hostname: &Hostname,
        identifier: &str,
    ) -> Result<Option<PairedDevice>, StorageErrors> {
        sqlx::query_as::<_, PairedDeviceRaw>(
            " INSERT OR IGNORE INTO paired_devices (hostname, identifier) VALUES ($1, $2) RETURNING *;",
        )
        .bind(hostname.as_str())
        .bind(identifier)
        .fetch_optional(&self.pool)
        .await
        .map( | this | this.map(Into::into))
        .map_err(Into::into)
    }

    pub async fn trusted_host_overwrite(
        &self,
        hostname: &Hostname,
        identifier: &str,
    ) -> Result<PairedDevice, StorageErrors> {
        sqlx::query_as::<_, PairedDeviceRaw>(
            " INSERT INTO paired_devices (hostname, identifier) VALUES ($1, $2) ON CONFLICT (hostname) DO UPDATE SET identifier = excluded.identifier RETURNING *;",
        )
        .bind(hostname.as_str())
        .bind(identifier)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(Into::into)
    }

    pub async fn client_id(&self) -> Result<uuid::Uuid, StorageErrors> {
        let id = sqlx::query_as::<_, DeviceId>("SELECT device FROM device_id")
            .fetch_optional(&self.pool)
            .await?;

        if let Some(id) = id {
            return Ok(id.device);
        }

        let id = uuid::Uuid::new_v4();

        sqlx::query("INSERT INTO device_id (device) VALUES ($1);")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(id)
    }
}
