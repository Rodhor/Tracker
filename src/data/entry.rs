use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserializse)]
pub struct TimeEntry {
    pub id: Uuid,
    pub task_id: Uuid,
    pub started_at: string,
    pub ended_at: Option<String>,
    pub minutes: Option<i64>,
    pub notes: Option<String>,
}

impl TimeEntry {
    pub fn new(task_id: Uuid) -> Self {
        use chrono::Utc;
        Self {
            id: Uuid::new_v4(),
            task_id,
            started_at: Utc::now().to_string(),
            ended_at: None,
            minutes: None,
            notes: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.ended_at.is_none()
    }
}
