use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Enum to represent the status of a task
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

impl TaskStatus {
    // Returns the next status for a task - Circleling around one Done
    pub fn next(&self) -> TaskStatus {
        match self {
            TaskStatus::Todo => TaskStatus::InProgress,
            TaskStatus::InProgress => TaskStatus::Done,
            TaskStatus::Done => TaskStatus::Todo,
        }
    }

    // Returns the label for a task status for usage in UI
    pub fn label(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "To Do",
            TaskStatus::InProgress => "In Progress",
            TaskStatus::Done => "Done",
        }
    }
}

// --- Task Struct ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub urgent: bool,
    pub important: bool,
    pub created_at: String,
}

impl Task {
    // Create a new task with a given name
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            status: TaskStatus::Todo,
            urgent: false,
            important: false,
            created_at: Utc::now().to_string(),
        }
    }

    // Returns the quadrant for a task - 1 for urgent and important, 2 for not urgent and important, etc.
    // Used for sorting tasks in the UI with the Eisenhower matrix.
    pub fn quadrant(&self) -> u8 {
        match (self.urgent, self.important) {
            (true, true) => 1,
            (false, true) => 2,
            (true, false) => 3,
            (false, false) => 4,
        }
    }

    // Will be used to determine if a task has already been sorted or not.
    pub fn has_priority(&self) -> bool {
        self.urgent || self.important
    }
}
