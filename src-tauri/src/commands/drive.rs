use crate::core::ata::detect_ata;
use crate::core::crypto_erase::detect_crypto_capable;
use crate::core::drive_sanitize::{self, DriveSanitizeMode};
use crate::core::nvme::detect_nvme;
use crate::error::{Result, SecDelError};
use crate::utils;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub path: String,
    pub model: String,
    pub size: u64,
    pub interface_type: String,
    pub is_ssd: bool,
    pub is_nvme: bool,
    pub supports_trim: bool,
    pub supports_secure_erase: bool,
    pub supports_sanitize: bool,
    pub supports_crypto_erase: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDetails {
    pub path: String,
    pub model: String,
    pub size: u64,
    pub interface_type: String,
    pub is_ssd: bool,
    pub is_nvme: bool,
    pub supports_trim: bool,
    pub supports_secure_erase: bool,
    pub supports_sanitize: bool,
    pub supports_crypto_erase: bool,
    pub firmware_version: String,
    pub serial_number: String,
    pub nvme_version: String,
    pub tcg_opal_supported: bool,
    pub encryption_enabled: bool,
    pub temperature: Option<f64>,
    pub wear_level: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveEraseReport {
    pub device: String,
    pub mode: u8,
    pub method_used: String,
    pub bytes_erased: u64,
    pub success: bool,
    pub verification_passed: bool,
    pub start_time: String,
    pub end_time: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EraseProgress {
    pub current: u64,
    pub total: u64,
    pub phase: String,
}

fn to_sanitize_device(dev: &DeviceInfo) -> drive_sanitize::DeviceInfo {
    drive_sanitize::DeviceInfo {
        path: dev.path.clone(),
        model: dev.model.clone(),
        size: dev.size,
        interface_type: dev.interface_type.clone(),
        is_ssd: dev.is_ssd,
        is_nvme: dev.is_nvme,
        supports_trim: dev.supports_trim,
        supports_secure_erase: dev.supports_secure_erase,
        supports_sanitize: dev.supports_sanitize,
        supports_crypto_erase: dev.supports_crypto_erase,
    }
}

#[tauri::command]
pub async fn get_devices() -> Result<Vec<DeviceInfo>> {
    let devices = crate::platform::wmi::query_wmi("SELECT * FROM Win32_DiskDrive")?;

    let mut result = Vec::new();
    for device in &devices {
        let model = device
            .get("Model")
            .and_then(|v| v.clone())
            .unwrap_or_default()
            .trim_matches('\0')
            .to_string();
        let device_id = device
            .get("DeviceID")
            .and_then(|v| v.clone())
            .unwrap_or_default()
            .trim_matches('\0')
            .to_string();
        let size_str = device
            .get("Size")
            .and_then(|v| v.clone())
            .unwrap_or_default()
            .trim_matches('\0')
            .to_string();
        let size = size_str.parse::<u64>().unwrap_or(0);
        let interface_type = device
            .get("InterfaceType")
            .and_then(|v| v.clone())
            .unwrap_or_default()
            .trim_matches('\0')
            .to_string();
        let media_type = device
            .get("MediaType")
            .and_then(|v| v.clone())
            .unwrap_or_default()
            .trim_matches('\0')
            .to_string();
        let pnp_device_id = device
            .get("PNPDeviceID")
            .and_then(|v| v.clone())
            .unwrap_or_default()
            .trim_matches('\0')
            .to_string();

        let is_nvme = interface_type.contains("NVMe")
            || pnp_device_id.contains("NVMe")
            || (interface_type.contains("SCSI") && pnp_device_id.contains("NVMe"));
        let is_ssd =
            is_nvme || media_type.contains("SSD") || media_type.contains("Fixed hard disk media");
        let device_path = format!(
            "\\\\.\\PHYSICALDRIVE{}",
            device_id.trim_start_matches(r"\\.\PHYSICALDRIVE")
        );

        let nvme_info = detect_nvme(&device_path).ok().flatten();
        let ata_info = detect_ata(&device_path).ok().flatten();
        let crypto_info = detect_crypto_capable(&device_path).ok().flatten();

        result.push(DeviceInfo {
            path: device_path,
            model,
            size,
            interface_type: if is_nvme { "NVMe".into() } else { "ATA".into() },
            is_ssd,
            is_nvme,
            supports_trim: is_ssd,
            supports_secure_erase: ata_info
                .as_ref()
                .map(|a| a.secure_erase_supported)
                .unwrap_or(false)
                || nvme_info
                    .as_ref()
                    .map(|n| n.secure_erase_supported)
                    .unwrap_or(false),
            supports_sanitize: nvme_info
                .as_ref()
                .map(|n| n.sanitize_supported)
                .unwrap_or(false),
            supports_crypto_erase: crypto_info
                .as_ref()
                .map(|c| c.tcg_opal_supported || c.hardware_encryption)
                .unwrap_or(false),
        });
    }
    Ok(result)
}

#[tauri::command]
pub async fn get_device_info(path: String) -> Result<DeviceDetails> {
    let devices = get_devices().await?;
    let base = devices
        .into_iter()
        .find(|d| d.path == path)
        .ok_or_else(|| SecDelError::DeviceNotFound(format!("Device {} not found", path)))?;

    let nvme_info = detect_nvme(&path).ok().flatten();
    let crypto_info = detect_crypto_capable(&path).ok().flatten();

    let nvme_version = nvme_info
        .as_ref()
        .map(|n| n.version.clone())
        .unwrap_or_default();
    let temperature = nvme_info.as_ref().and_then(|n| n.temperature);
    let wear_level = nvme_info.as_ref().and_then(|n| n.wear_level);

    Ok(DeviceDetails {
        path: base.path,
        model: base.model,
        size: base.size,
        interface_type: base.interface_type,
        is_ssd: base.is_ssd,
        is_nvme: base.is_nvme,
        supports_trim: base.supports_trim,
        supports_secure_erase: base.supports_secure_erase,
        supports_sanitize: base.supports_sanitize,
        supports_crypto_erase: base.supports_crypto_erase,
        firmware_version: String::new(),
        serial_number: String::new(),
        nvme_version,
        tcg_opal_supported: crypto_info
            .as_ref()
            .map(|c| c.tcg_opal_supported)
            .unwrap_or(false),
        encryption_enabled: crypto_info
            .as_ref()
            .map(|c| c.encryption_enabled)
            .unwrap_or(false),
        temperature,
        wear_level,
    })
}

#[tauri::command]
pub async fn erase_drive(
    app_handle: AppHandle,
    device: String,
    mode: u8,
    state: State<'_, AppState>,
) -> Result<DriveEraseReport> {
    if !utils::is_admin() {
        return Err(SecDelError::AdminRequired(
            "Admin privileges required for drive erasure".into(),
        ));
    }

    let devices = get_devices().await?;
    let dev_info = devices
        .into_iter()
        .find(|d| d.path == device)
        .ok_or_else(|| SecDelError::DeviceNotFound(format!("Device {} not found", device)))?;

    if state.cancelled.load(Ordering::Relaxed) {
        state.cancelled.store(false, Ordering::Relaxed);
        return Err(SecDelError::OperationCancelled);
    }

    app_handle
        .emit(
            "erase_progress",
            EraseProgress {
                current: 0,
                total: 100,
                phase: "starting".into(),
            },
        )
        .ok();

    let start_time = utils::timestamp_iso8601();
    let sanitize_mode = match mode {
        0 => DriveSanitizeMode::Quick,
        1 => DriveSanitizeMode::Standard,
        2 => DriveSanitizeMode::Secure,
        _ => DriveSanitizeMode::Ultimate,
    };

    let sanitize_dev = to_sanitize_device(&dev_info);
    let report = drive_sanitize::sanitize_drive(&sanitize_dev, sanitize_mode)?;

    let end_time = utils::timestamp_iso8601();

    app_handle
        .emit(
            "erase_progress",
            EraseProgress {
                current: 100,
                total: 100,
                phase: "complete".into(),
            },
        )
        .ok();

    Ok(DriveEraseReport {
        device,
        mode,
        method_used: report.method_used,
        bytes_erased: report.bytes_erased,
        success: report.success,
        verification_passed: report.verification_passed,
        start_time,
        end_time,
        details: report.details,
    })
}
