use super::entry::TimeEntry;
use super::task::Task;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppData {
    #[serde(default)]
    pub tasks: Vec<Task>,

    #[serde(default)]
    pub entries: Vec<TimeEntry>,
}

// Data is persisted to ~/.tracker/data.json as pretty-printed JSON.

pub fn data_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = home.join(".tracker");
    let _ = fs::create_dir_all(&dir);
    dir.join("data.json")
}

pub fn load_data() -> Result<AppData, String> {
    let path = data_path();
    if !path.exists() {
        return Ok(AppData::default());
    }
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read data file: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse data file: {e}"))
}

pub fn save_data(data: &AppData) -> Result<(), String> {
    let path = data_path();
    let content =
        serde_json::to_string_pretty(data).map_err(|e| format!("Failed to serialize data: {e}"))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write data file: {e}"))
}
