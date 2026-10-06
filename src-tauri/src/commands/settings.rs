use crate::error::Result;
use crate::utils;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: String,
    pub language: String,
    pub overwrite_passes: u32,
    pub verify_writes: bool,
    pub trim_enabled: bool,
    pub auto_report: bool,
    pub notification_level: String,
    pub admin_mode: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            language: "en".into(),
            overwrite_passes: 3,
            verify_writes: true,
            trim_enabled: true,
            auto_report: false,
            notification_level: "info".into(),
            admin_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub version: String,
    pub admin: bool,
    pub is_running: bool,
    pub uptime_seconds: u64,
    pub last_operation: String,
    pub total_files_deleted: u64,
    pub total_bytes_erased: u64,
}

fn settings_path() -> PathBuf {
    let mut path = dirs_next().unwrap_or_else(|| PathBuf::from("."));
    path.push("secdel_settings.json");
    path
}

fn dirs_next() -> Option<PathBuf> {
    std::env::current_dir().ok()
}

#[tauri::command]
pub async fn get_settings() -> Result<Settings> {
    let path = settings_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| crate::error::SecDelError::Io(e.to_string()))?;
        let settings: Settings = serde_json::from_str(&content)
            .map_err(|e| crate::error::SecDelError::Serialization(e.to_string()))?;
        Ok(settings)
    } else {
        Ok(Settings::default())
    }
}

#[tauri::command]
pub async fn save_settings(settings: Settings) -> Result<()> {
    let path = settings_path();
    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| crate::error::SecDelError::Serialization(e.to_string()))?;
    std::fs::write(&path, content).map_err(|e| crate::error::SecDelError::Io(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn get_app_status() -> Result<AppStatus> {
    Ok(AppStatus {
        version: "1.0.0".into(),
        admin: utils::is_admin(),
        is_running: true,
        uptime_seconds: 0,
        last_operation: "none".into(),
        total_files_deleted: 0,
        total_bytes_erased: 0,
    })
}
