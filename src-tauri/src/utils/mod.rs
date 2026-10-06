use sha2::{Digest, Sha256};
use std::path::Path;

pub fn hash_file(path: &Path) -> Result<String, std::io::Error> {
    let data = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

pub fn file_size_human(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let size = bytes as f64;
    let unit_idx = if size < 1.0 {
        0
    } else {
        (size.log10() / 3.0) as usize
    };
    let unit_idx = unit_idx.min(UNITS.len() - 1);
    let value = size / (1024u64.pow(unit_idx as u32) as f64);
    format!("{:.2} {}", value, UNITS[unit_idx])
}

pub fn is_admin() -> bool {
    #[cfg(windows)]
    {
        unsafe {
            let mut token_handle = windows::Win32::Foundation::HANDLE::default();
            let result = windows::Win32::System::Threading::OpenProcessToken(
                windows::Win32::System::Threading::GetCurrentProcess(),
                windows::Win32::Security::TOKEN_QUERY,
                &mut token_handle,
            );
            if result.is_err() {
                return false;
            }
            let mut elevation: u32 = 0;
            let mut return_length: u32 = 0;
            let info_result = windows::Win32::Security::GetTokenInformation(
                token_handle,
                windows::Win32::Security::TokenElevation,
                Some(&mut elevation as *mut _ as *mut std::ffi::c_void),
                std::mem::size_of::<u32>() as u32,
                &mut return_length,
            );
            windows::Win32::Foundation::CloseHandle(token_handle).ok();
            info_result.is_ok() && elevation != 0
        }
    }
    #[cfg(not(windows))]
    {
        false
    }
}

pub fn timestamp_iso8601() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub fn timestamp_human() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
