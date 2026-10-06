use crate::commands::report::ReportData;
use crate::error::{Result, SecDelError};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn generate_json_report(data: &ReportData, output_path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| SecDelError::ReportFailed(format!("JSON serialization error: {}", e)))?;

    let mut file = File::create(output_path)
        .map_err(|e| SecDelError::ReportFailed(format!("Cannot create JSON file: {}", e)))?;

    file.write_all(json.as_bytes())
        .map_err(|e| SecDelError::ReportFailed(format!("JSON write error: {}", e)))?;

    file.flush()
        .map_err(|e| SecDelError::ReportFailed(format!("JSON flush error: {}", e)))?;

    Ok(())
}
