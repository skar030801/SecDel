use crate::commands::report::ReportData;
use crate::error::{Result, SecDelError};
use std::fs::File;
use std::path::Path;

pub fn generate_csv_report(data: &ReportData, output_path: &Path) -> Result<()> {
    let file = File::create(output_path)
        .map_err(|e| SecDelError::ReportFailed(format!("Cannot create CSV file: {}", e)))?;

    let mut writer = csv::Writer::from_writer(file);

    let empty: [&str; 0] = [];

    writer
        .write_record(&["Section", "Field", "Value"])
        .map_err(|e| SecDelError::ReportFailed(format!("CSV write error: {}", e)))?;

    writer
        .write_record(&["Report", "Timestamp", &data.timestamp])
        .map_err(|e| SecDelError::ReportFailed(format!("CSV write error: {}", e)))?;
    writer
        .write_record(&["Report", "Application Version", &data.application_version])
        .map_err(|e| SecDelError::ReportFailed(format!("CSV write error: {}", e)))?;
    writer
        .write_record(&["Report", "Operation Type", &data.operation_type])
        .map_err(|e| SecDelError::ReportFailed(format!("CSV write error: {}", e)))?;
    writer
        .write_record(&["Report", "Level", &data.level.to_string()])
        .map_err(|e| SecDelError::ReportFailed(format!("CSV write error: {}", e)))?;
    writer
        .write_record(&["Report", "Total Files", &data.total_files.to_string()])
        .map_err(|e| SecDelError::ReportFailed(format!("CSV write error: {}", e)))?;
    writer
        .write_record(&[
            "Report",
            "Total Bytes",
            &data.total_bytes_processed.to_string(),
        ])
        .map_err(|e| SecDelError::ReportFailed(format!("CSV write error: {}", e)))?;
    writer
        .write_record(&[
            "Report",
            "Confidence Score",
            &format!("{:.1}%", data.confidence_score),
        ])
        .map_err(|e| SecDelError::ReportFailed(format!("CSV write error: {}", e)))?;
    writer
        .write_record(&[
            "Report",
            "Duration (s)",
            &format!("{:.1}", data.duration_seconds),
        ])
        .map_err(|e| SecDelError::ReportFailed(format!("CSV write error: {}", e)))?;

    writer
        .write_record(&empty)
        .map_err(|e| SecDelError::ReportFailed(format!("CSV write error: {}", e)))?;
    writer
        .write_record(&[
            "Files",
            "Path",
            "Size",
            "Overwrite Passes",
            "Trimmed",
            "Verified",
            "Confidence",
        ])
        .map_err(|e| SecDelError::ReportFailed(format!("CSV write error: {}", e)))?;

    for file in &data.files {
        writer
            .write_record(&[
                "Files",
                &file.path,
                &file.size.to_string(),
                &file.overwrite_passes.to_string(),
                if file.trimmed { "Yes" } else { "No" },
                if file.verified { "Yes" } else { "No" },
                &format!("{:.1}%", file.confidence),
            ])
            .map_err(|e| SecDelError::ReportFailed(format!("CSV write error: {}", e)))?;
    }

    writer
        .write_record(&empty)
        .map_err(|e| SecDelError::ReportFailed(format!("CSV write error: {}", e)))?;
    writer
        .write_record(&["Devices", "Model", "Erase Method", "Success"])
        .map_err(|e| SecDelError::ReportFailed(format!("CSV write error: {}", e)))?;

    for device in &data.devices {
        writer
            .write_record(&[
                "Devices",
                &device.model,
                &device.erase_method,
                if device.success { "Yes" } else { "No" },
            ])
            .map_err(|e| SecDelError::ReportFailed(format!("CSV write error: {}", e)))?;
    }

    writer
        .flush()
        .map_err(|e| SecDelError::ReportFailed(format!("CSV flush error: {}", e)))?;

    Ok(())
}
