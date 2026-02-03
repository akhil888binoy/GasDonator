
// src/services/activity_monitor.rs
use crate::{
    error::error::AppError,
    suspicious_activities::suspicious_activity::SuspiciousActivityRepo,
};
use serde_json::json;


#[derive(Clone)]
pub struct ActivityMonitor {
    repo: SuspiciousActivityRepo,
}

impl ActivityMonitor {
    pub fn new(repo: SuspiciousActivityRepo) -> Self {
        Self { repo }
    }

    pub async fn log_failed_login(
        &self,
        user_id: String,
        reason: &str,
    ) -> Result<(), AppError> {
        self.repo.log_activity(
            user_id,
            "failed_login".to_string(),
            "medium".to_string(),
            json!({ "reason": reason }),
        ).await
    }

    pub async fn log_failed_deposit(
        &self,
        user_id: &str,
        tx_hash: &str,
        reason: &str,
    ) -> Result<(), AppError> {
        self.repo.log_activity(
            user_id.to_string(),
            "failed_deposit".to_string(),
            "high".to_string(),
            json!({
                "tx_hash": tx_hash,
                "reason": reason
            }),
        ).await
    }

    pub async fn log_failed_withdrawal(
        &self,
        user_id: &str,
        amount: &str,
        token: &str,
        reason: &str,
    ) -> Result<(), AppError> {
        self.repo.log_activity(
            user_id.to_string(),
            "failed_withdrawal".to_string(),
            "high".to_string(),
            json!({
                "amount": amount,
                "token": token,
                "reason": reason
            }),
        ).await
    }

    pub async fn log_unusual_activity(
        &self,
        user_id: String,
        activity_type: &str,
        details: serde_json::Value,
    ) -> Result<(), AppError> {
        self.repo.log_activity(
            user_id,
            activity_type.to_string(),
            "medium".to_string(),
            details,
        ).await
    }
}