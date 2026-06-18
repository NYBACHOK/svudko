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

use crate::{request::StorageRequest, setup::setup_db};

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
        todo!()
    }
}
