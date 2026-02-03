

use crate::error::error::AppError;
use crate::state_models::models::DbConnection;
use crate::{jobs::gasdonator::donate_wallet, suspicious_activities::activity_monitor::ActivityMonitor};
use std::time::Duration;  
use sea_orm::ConnectionTrait;
use tokio;
use tokio::time::sleep;
use tracing::{error, info, warn};

const WITHDRAW_INTERVAL: Duration = Duration::from_secs(2);
pub const MAX_RETRIES: u32 = 1;
pub const RETRY_BACKOFF: Duration = Duration::from_secs(1);

pub async fn run_donator(
    worker_id: u64,
    db: DbConnection,
) ->  Result<(), AppError> {
    println!("WORKER:{} RUNNING", worker_id);
    loop {
        let mut retries = 0;
        let result = loop {

            match donate_wallet(worker_id, &db ).await {
                Ok(_) => break Ok(()),
                Err(e) if retries >= MAX_RETRIES => break Err(e),
                Err(e) => {
                    warn!("Price submit failed (attempt {}): {}", retries + 1, e);
                    retries += 1;
                    sleep(RETRY_BACKOFF * retries).await;
                }
            }
        };

        match result {
            Ok(_) => info!("Successfully submitted prices at {}", chrono::Local::now()),
            Err(e) => error!("Fatal error submitting prices: {}", e),
        }

        sleep(WITHDRAW_INTERVAL).await;

        between_cycles_cleanup(&db).await;

        tokio::task::yield_now().await;
    }
}



pub async fn between_cycles_cleanup(db: &DbConnection) {
    // 1. Check connection health by executing a simple query
    if let Err(e) = db.0.execute_unprepared("SELECT 1").await {
        warn!("Database connection health check failed: {}", e);
    }

    // 2. Log memory statistics (Linux-specific)
    if cfg!(target_os = "linux") {
        if let Ok(usage) = get_memory_usage() {
            info!("Memory usage: {}MB resident", usage / 1024 / 1024);
        }
    }

    // 3. Force Tokio to reclaim resources
    tokio::task::consume_budget().await;
}


fn get_memory_usage() -> Result<usize, std::io::Error> {
    // Read from /proc/self/statm
    // Format: total_size resident_size shared_size text_size lib_data_size ...
    let statm = std::fs::read_to_string("/proc/self/statm")?;
    
    statm.split_whitespace()
        .nth(1) // Get resident memory size (second field)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to parse memory info"
        ))
}