use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

impl TaskStatus {
    // Cycles: Todo → InProgress → Done → Todo
    pub fn next(&self) -> TaskStatus {
        match self {
            TaskStatus::Todo => TaskStatus::InProgress,
            TaskStatus::InProgress => TaskStatus::Done,
            TaskStatus::Done => TaskStatus::Todo,
        }
    }

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
    #[serde(default)]
    pub completed_at: Option<String>,
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
            created_at: Utc::now().to_rfc3339(),
            completed_at: None,
        }
    }

    // Eisenhower quadrant: 1 = urgent+important, 2 = important only, 3 = urgent only, 4 = neither
    pub fn quadrant(&self) -> u8 {
        match (self.urgent, self.important) {
            (true, true) => 1,
            (false, true) => 2,
            (true, false) => 3,
            (false, false) => 4,
        }
    }

    // True once at least one priority flag is set — separates sorted from unsorted in the list
    pub fn has_priority(&self) -> bool {
        self.urgent || self.important
    }

    pub fn complete(&mut self) {
        let now = Utc::now().to_rfc3339();
        self.completed_at = Some(now);
    }

    pub fn completed_today(&self) -> bool {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        self.completed_at
            .as_deref()
            .map(|s| s.starts_with(&today))
            .unwrap_or(false)
    }
}
impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}
