use crate::{
    entities::suspicious_activities,
    error::error::AppError
};
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, 
    ActiveValue::Set, QueryFilter, ColumnTrait
};

use chrono::Utc;

#[derive(Clone)]
pub struct SuspiciousActivityRepo {
    db: DatabaseConnection,
}

impl SuspiciousActivityRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn log_activity(
        &self,
        user_id: String,
        activity_type: String,
        severity: String,
        details: serde_json::Value,
    ) -> Result<(), AppError> {
        let activity = suspicious_activities::ActiveModel {
            user_id: Set(user_id),
            activity_type: Set(activity_type),
            severity: Set(severity),
            details: Set(details),
            resolved: Set(false),
            resolved_at: Set(None),
            ..Default::default()
        };

        activity.insert(&self.db).await?;
        Ok(())
    }

    pub async fn mark_as_resolved(
        &self,
        activity_id: i32,
    ) -> Result<(), AppError> {
        let activity = suspicious_activities::Entity::find_by_id(activity_id)
            .one(&self.db)
            .await?;

        match activity{
            Some(activity)=>{
                let mut activity: suspicious_activities::ActiveModel = activity.into();
                activity.resolved = Set(true);
                activity.resolved_at = Set(Some(Utc::now().fixed_offset()));
                activity.update(&self.db).await?;
            }
            None=>{
                    return Err(AppError::BadRequest("Activity not found".to_string()));
            }
        }
       
        Ok(())
    }

    pub async fn get_unresolved_activities(
        &self,
    ) -> Result<Vec<suspicious_activities::Model>, AppError> {
        suspicious_activities::Entity::find()
            .filter(suspicious_activities::Column::Resolved.eq(false))
            .all(&self.db)
            .await
            .map_err(Into::into)
    }
}