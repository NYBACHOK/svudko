use std::path::Path;

use crate::{APPLY_MIGRATIONS, ConnectError, database_path};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./sql/");

async fn connect(
    database_location: impl AsRef<Path>,
    migrate: bool,
) -> Result<sqlx::SqlitePool, ConnectError> {
    let opt = sqlx::sqlite::SqliteConnectOptions::new()
        .create_if_missing(true)
        .read_only(false)
        .filename(database_location);

    let pool = sqlx::SqlitePool::connect_with(opt).await?;

    if migrate {
        MIGRATOR.run(&pool).await?;
    }

    Ok(pool)
}

pub async fn setup_db() -> Result<sqlx::SqlitePool, ConnectError> {
    let database_location = database_path();

    tracing::debug!("Database location: {}", database_location.to_string_lossy());

    let db_pool = connect(&database_location, APPLY_MIGRATIONS).await;
    if let Err(ConnectError::Migration(_)) = db_pool {
        #[cfg(debug_assertions)]
        {
            std::fs::remove_file(&database_location)?;
            tracing::warn!("Database deleted. Creating new");
        }

        connect(database_location, APPLY_MIGRATIONS).await
    } else {
        db_pool
    }
}
