
use sea_orm::Database;

use crate::{config::config::AppConfig, error::error::AppError, state_models::models::DbConnection};




pub async fn init_db()->Result<DbConnection, AppError>{ 

    let config = AppConfig::from_env()?;
    let db_url = &config.database_url;
    let db = Database::connect(db_url)
        .await
        .map_err(|e| AppError::DbError(e))?;

    Ok(DbConnection(db))

}