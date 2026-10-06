use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtaInfo {
    pub secure_erase_supported: bool,
    pub enhanced_secure_erase_supported: bool,
    pub is_frozen: bool,
    pub is_ssd: bool,
    pub model: String,
    pub firmware: String,
    pub serial: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtaResult {
    pub success: bool,
    pub method: String,
    pub duration_seconds: f64,
    pub details: String,
}

pub fn detect_ata(device_path: &str) -> Result<Option<AtaInfo>> {
    let result = crate::platform::ata::query_ata_info(device_path)?;

    match result {
        Some(info) => Ok(Some(AtaInfo {
            secure_erase_supported: info.secure_erase_supported,
            enhanced_secure_erase_supported: info.enhanced_erase_supported,
            is_frozen: info.is_frozen,
            is_ssd: info.is_ssd,
            model: info.model,
            firmware: info.firmware,
            serial: info.serial,
        })),
        None => Ok(None),
    }
}

pub fn ata_secure_erase(device_path: &str, enhanced: bool) -> Result<AtaResult> {
    let start = std::time::Instant::now();

    let identify = crate::platform::ata::read_identify_data(device_path)?;

    let use_enhanced = enhanced && identify.enhanced_erase_supported;

    crate::platform::ata::security_set_password(device_path, b"SecDelErase2024!")?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    crate::platform::ata::security_erase_unit(device_path, use_enhanced)?;

    let duration = start.elapsed().as_secs_f64();

    let method = if use_enhanced {
        "ATA Enhanced Secure Erase"
    } else {
        "ATA Secure Erase"
    };

    Ok(AtaResult {
        success: true,
        method: method.into(),
        duration_seconds: duration,
        details: format!("{} completed in {:.1}s", method, duration),
    })
}
