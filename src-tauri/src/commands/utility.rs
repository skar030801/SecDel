use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[tauri::command]
pub fn get_file_size(path: String) -> Result<u64> {
    let meta = std::fs::metadata(&path)?;
    Ok(meta.len())
}

#[tauri::command]
pub fn get_recent_logs(_limit: u32) -> Vec<LogEvent> {
    Vec::new()
}

#[tauri::command]
pub fn get_default_settings() -> HashMap<String, serde_json::Value> {
    let mut settings = HashMap::new();
    settings.insert("theme".into(), serde_json::Value::String("dark".into()));
    settings.insert(
        "defaultSecurityLevel".into(),
        serde_json::Value::Number(2.into()),
    );
    settings.insert(
        "overwritePasses".into(),
        serde_json::Value::Number(3.into()),
    );
    settings.insert("verifyAfterDelete".into(), serde_json::Value::Bool(true));
    settings.insert("trimAfterDelete".into(), serde_json::Value::Bool(true));
    settings.insert("autoGenerateReport".into(), serde_json::Value::Bool(false));
    settings.insert(
        "notificationLevel".into(),
        serde_json::Value::String("info".into()),
    );
    settings.insert("adminMode".into(), serde_json::Value::Bool(false));
    settings
}

#[tauri::command]
pub fn export_scan_results() -> Result<String> {
    let path = std::env::temp_dir().join("secdel_scan_results.json");
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn download_report() -> Result<String> {
    Ok("Report download started".into())
}

#[tauri::command]
pub fn open_report() -> Result<String> {
    Ok("Report opened".into())
}
