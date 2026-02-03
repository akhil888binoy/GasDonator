
use crate::{ db::connection::init_db, error::error::AppError,  jobs::index::{run_donator}, suspicious_activities::{activity_monitor::ActivityMonitor, suspicious_activity::SuspiciousActivityRepo}};
pub mod db;
pub mod error;
pub mod config;
pub mod state_models;
pub mod chain_config;
pub mod jobs;
pub mod entities;
pub mod suspicious_activities;
pub mod tokens;


#[actix_web::main] 
async fn main() -> Result<(), AppError> {
    // Initialize logging first
    
    let db = init_db().await
        .map_err(|e| {
            tracing::error!("Database initialization failed: {}", e);
            AppError::InternalError(format!("DB init error: {}", e))
        })?;
    
    
    // Use join_all to wait for all workers
    let workers = vec![
        tokio::spawn(run_donator(0, db.clone())),
        tokio::spawn(run_donator(1, db.clone())),
        tokio::spawn(run_donator(2, db.clone())),
        tokio::spawn(run_donator(3, db.clone())),
    ];
    
    // Wait for all workers (they should run forever unless error)
    for worker in workers {
        if let Err(e) = worker.await {
            tracing::error!("Worker failed: {:?}", e);
        }
    }
    
    tracing::info!("All workers stopped");
    Ok(())
}

