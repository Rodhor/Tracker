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

// Create path to data file
pub fn data_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = home.join(".tracker");
    let _ = fs::create_dir_all(&dir);
    dir.join("data.json")
}

pub fn load_data() -> Result<AppData, string> {
    // Retrieve the path
    let path = data_path();

    // If the file does not exist, return an empty AppData
    if !path.exists() {
        return Ok(AppData::default());
    }

    // Try to read the file content, return an error as string if it fails
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read data file: {e}"))?;

    // Try to parse the content into an AppData struct, return an error as string if it fails
    let data: AppData =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse data file: {e}"))?;

    // If successful, return the parsed data
    Ok(data)
}

pub fn save_data(data: &AppData) -> Result<(), string> {
    // Retrieve the path
    let path = data_path();

    // Try to serialize the data into a JSON string, return an error as string if it fails
    let content =
        serde_json::to_string_pretty(data).map_err(|e| format!("Failed to serialize data: {e}"))?;

    // Try to write the serialized data to the file, return an error as string if it fails
    fs::write(&path, content).map_err(|e| format!("Failed to write data file: {e}"))?;

    // If successful, return empty Ok value
    Ok(())
}
