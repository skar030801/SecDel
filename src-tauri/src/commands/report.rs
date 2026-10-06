use crate::error::{Result, SecDelError};
use crate::reporting;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub overwrite_passes: u32,
    pub trimmed: bool,
    pub verified: bool,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceEntry {
    pub path: String,
    pub model: String,
    pub interface_type: String,
    pub erase_method: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportData {
    pub timestamp: String,
    pub application_version: String,
    pub operation_type: String,
    pub level: u8,
    pub files: Vec<FileEntry>,
    pub devices: Vec<DeviceEntry>,
    pub total_bytes_processed: u64,
    pub total_files: u32,
    pub total_devices: u32,
    pub confidence_score: f64,
    pub duration_seconds: f64,
    pub notes: String,
}

#[tauri::command]
pub async fn generate_report(
    report_data: ReportData,
    format: String,
    output_path: String,
) -> Result<String> {
    let path = Path::new(&output_path);

    match format.to_lowercase().as_str() {
        "pdf" => {
            reporting::pdf::generate_pdf_report(&report_data, path)?;
        }
        "csv" => {
            reporting::csv::generate_csv_report(&report_data, path)?;
        }
        "json" => {
            reporting::json::generate_json_report(&report_data, path)?;
        }
        _ => {
            return Err(SecDelError::InvalidParameter(format!(
                "Unsupported report format: {}",
                format
            )));
        }
    }

    Ok(output_path)
}
