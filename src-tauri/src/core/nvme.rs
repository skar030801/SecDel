use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvmeInfo {
    pub version: String,
    pub sanitize_supported: bool,
    pub format_nvm_supported: bool,
    pub secure_erase_supported: bool,
    pub namespace_count: u32,
    pub temperature: Option<f64>,
    pub wear_level: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NvmeSanitizeAction {
    BlockErase,
    Overwrite,
    CryptoErase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvmeResult {
    pub success: bool,
    pub method: String,
    pub duration_seconds: f64,
    pub details: String,
}

pub fn detect_nvme(device_path: &str) -> Result<Option<NvmeInfo>> {
    let result = crate::platform::nvme::query_nvme_info(device_path)?;

    match result {
        Some(info) => Ok(Some(NvmeInfo {
            version: info.version,
            sanitize_supported: info.sanitize_capable,
            format_nvm_supported: info.format_nvm_supported,
            secure_erase_supported: info.secure_erase_supported,
            namespace_count: info.namespace_count,
            temperature: info.temperature,
            wear_level: info.wear_level,
        })),
        None => Ok(None),
    }
}

pub fn nvme_sanitize(device_path: &str, action: NvmeSanitizeAction) -> Result<NvmeResult> {
    let action_str = match action {
        NvmeSanitizeAction::BlockErase => "Block Erase",
        NvmeSanitizeAction::Overwrite => "Overwrite",
        NvmeSanitizeAction::CryptoErase => "Crypto Erase",
    };

    let action_code: u8 = match action {
        NvmeSanitizeAction::BlockErase => 0x02,
        NvmeSanitizeAction::Overwrite => 0x03,
        NvmeSanitizeAction::CryptoErase => 0x04,
    };

    let start = std::time::Instant::now();

    crate::platform::nvme::send_sanitize_command(device_path, action_code)?;

    let duration = start.elapsed().as_secs_f64();

    Ok(NvmeResult {
        success: true,
        method: format!("NVMe Sanitize - {}", action_str),
        duration_seconds: duration,
        details: format!(
            "NVMe sanitize ({}) completed in {:.1}s",
            action_str, duration
        ),
    })
}

pub fn nvme_format_nvm(device_path: &str, secure_erase: bool) -> Result<NvmeResult> {
    let start = std::time::Instant::now();

    crate::platform::nvme::send_format_nvm_command(device_path, secure_erase)?;

    let duration = start.elapsed().as_secs_f64();

    Ok(NvmeResult {
        success: true,
        method: if secure_erase {
            "NVMe Format NVM with Secure Erase".into()
        } else {
            "NVMe Format NVM".into()
        },
        duration_seconds: duration,
        details: format!("NVMe Format NVM completed in {:.1}s", duration),
    })
}
