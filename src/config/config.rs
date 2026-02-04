use std::env;

use dotenv::dotenv;
use serde::Deserialize;

use crate::error::error::AppError;


/// Configuration settings for our application.
///
/// This struct holds all the essential configuration values, primarily loaded
/// from environment variables. Think of it as the central place where we define
/// what our app needs to know about its environment to run correctly,
/// like database connections and API keys.
///
/// We `derive` `Debug` for easy printing during development/debugging,
/// `Deserialize` because it's good practice for configs (even if we're not
/// deserializing from a file directly here), and `Clone` so we can pass
/// copies of the config around safely without moving ownership.
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    
    /// The full database connection URL. This tells SeaORM how to connect
    /// to our PostgreSQL (or other) database.
    pub database_url: String,
    /// The address (IP:Port) where our Actix-Web server will listen for incoming requests.
    /// Defaults to "127.0.0.1:8080" if not explicitly set in the environment.
    ///Private Key of signer who signs the contract
    pub signer0: String ,

    pub signer1: String ,

    pub signer2: String,

    pub signer3: String,

    pub master_wallet_address : String ,

}



/// Attempts to load the application configuration from environment variables.
///
/// This is our primary way to get the app's settings. It first tries to
/// load variables from a `.env` file (super handy for local development!)
/// and then fetches specific variables.
///
/// If any critical variable (like `SUPABASE_URL` or `DATABASE_URL`) is missing,
/// it gracefully returns an `AppError::ConfigError`, making it clear what's wrong.
///
/// # Returns
/// `Result<Self, AppError>`:
/// - `Ok(AppConfig)` if all required environment variables are found and parsed.
/// - `Err(AppError::ConfigError)` if any required variable is missing or there's an issue.
impl AppConfig {
    pub fn from_env() -> Result<Self, AppError> {
        dotenv().ok();
        Ok(AppConfig { 
            database_url: env::var("DATABASE_URL")
                .map_err(|e| AppError::ConfigError(format!("DATABASE_URL not set: {}", e)))?,
            signer0:env::var("SIGNER0")
                .map_err(|e| AppError::ConfigError(format!("SIGNER0 not set: {}", e)))?,
            signer1:env::var("SIGNER1")
                .map_err(|e| AppError::ConfigError(format!("SIGNER1 not set: {}", e)))?,
            signer2:env::var("SIGNER2")
                .map_err(|e| AppError::ConfigError(format!("SIGNER2 not set: {}", e)))?,
            signer3:env::var("SIGNER3")
                .map_err(|e| AppError::ConfigError(format!("SIGNER3 not set: {}", e)))?,
            master_wallet_address:env::var("MASTER_WALLET_ADDRESS")
                .map_err(|e| AppError::ConfigError(format!("MASTER_WALLET_ADDRESS not set: {}", e)))?,
        })
    }
}