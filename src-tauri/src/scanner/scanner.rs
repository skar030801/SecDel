use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::Win32::Storage::FileSystem::{
    FindClose, FindFirstStreamW, FindNextStreamW, STREAM_INFO_LEVELS,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactFinding {
    pub path: String,
    pub size: u64,
    pub risk_level: String,
    pub description: String,
}

#[repr(C, packed)]
struct Win32FindStreamData {
    stream_size: i64,
    stream_alloc_size: i64,
    stream_name: [u16; 260 + 36],
}

pub fn scan_alternate_data_streams(path: &Path) -> Vec<ArtifactFinding> {
    let mut findings = Vec::new();
    if !path.exists() {
        return findings;
    }

    let wide_path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let mut stream_data: Win32FindStreamData = std::mem::zeroed();
        let handle = match FindFirstStreamW(
            windows::core::PCWSTR(wide_path.as_ptr()),
            STREAM_INFO_LEVELS(0),
            &mut stream_data as *mut _ as *mut std::ffi::c_void,
            Some(0),
        ) {
            Ok(h) => h,
            Err(_) => return findings,
        };

        loop {
            let name_buf: [u16; 296] = stream_data.stream_name;
            let stream_name = String::from_utf16_lossy(&name_buf);
            let stream_trimmed = stream_name.trim_matches('\0');
            if stream_trimmed.is_empty() || stream_trimmed == "::$DATA" {
                break;
            }
            let name_clean = stream_trimmed.trim_start_matches(':');
            findings.push(ArtifactFinding {
                path: format!("{}:{}", path.display(), name_clean),
                size: stream_data.stream_size as u64,
                risk_level: "medium".into(),
                description: format!(
                    "Alternate data stream: {} (size: {})",
                    name_clean,
                    crate::utils::file_size_human(stream_data.stream_size.max(0) as u64)
                ),
            });

            stream_data = std::mem::zeroed();
            if FindNextStreamW(handle, &mut stream_data as *mut _ as *mut std::ffi::c_void).is_err()
            {
                break;
            }
        }
        FindClose(handle).ok();
    }
    findings
}

pub fn scan_shadow_copies() -> Result<Vec<ArtifactFinding>> {
    let mut findings = Vec::new();
    if let Ok(volumes) = crate::platform::wmi::query_wmi("SELECT * FROM Win32_ShadowCopy") {
        for vol in &volumes {
            if let Some(device_obj) = vol.get("DeviceObject").and_then(|v| v.clone()) {
                let path = device_obj.trim_matches('\0').to_string();
                let desc = vol
                    .get("InstallDate")
                    .and_then(|v| v.clone())
                    .map(|d| format!("Volume Shadow Copy from: {}", d.trim_matches('\0')))
                    .unwrap_or_else(|| "Volume Shadow Copy".into());
                findings.push(ArtifactFinding {
                    path,
                    size: 0,
                    risk_level: "high".into(),
                    description: desc,
                });
            }
        }
    }
    Ok(findings)
}

pub fn scan_restore_points() -> Result<Vec<ArtifactFinding>> {
    let mut findings = Vec::new();
    let system_path = std::env::var("SYSTEMROOT").unwrap_or_else(|_| "C:\\Windows".into());
    let restore_dir = Path::new(&system_path).join("System32\\Restore");
    if restore_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&restore_dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        findings.push(ArtifactFinding {
                            path: entry.path().to_string_lossy().to_string(),
                            size: meta.len(),
                            risk_level: "high".into(),
                            description: "System Restore point file".into(),
                        });
                    }
                }
            }
        }
    }
    Ok(findings)
}

pub fn scan_browser_caches() -> Result<Vec<ArtifactFinding>> {
    let mut findings = Vec::new();
    let user_profile = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| "C:\\Users\\Default".into());

    let chrome_cache =
        Path::new(&user_profile).join("AppData\\Local\\Google\\Chrome\\User Data\\Default\\Cache");
    if chrome_cache.exists() {
        findings.push(ArtifactFinding {
            path: chrome_cache.to_string_lossy().to_string(),
            size: total_dir_size(&chrome_cache),
            risk_level: "medium".into(),
            description: "Google Chrome browser cache".into(),
        });
    }
    let chrome_code_cache = Path::new(&user_profile)
        .join("AppData\\Local\\Google\\Chrome\\User Data\\Default\\Code Cache");
    if chrome_code_cache.exists() {
        findings.push(ArtifactFinding {
            path: chrome_code_cache.to_string_lossy().to_string(),
            size: total_dir_size(&chrome_code_cache),
            risk_level: "medium".into(),
            description: "Google Chrome code cache".into(),
        });
    }
    let firefox_cache = Path::new(&user_profile).join("AppData\\Local\\Mozilla\\Firefox\\Profiles");
    if firefox_cache.exists() {
        if let Ok(entries) = std::fs::read_dir(&firefox_cache) {
            for entry in entries.flatten() {
                let cache2 = entry.path().join("cache2");
                if cache2.exists() {
                    findings.push(ArtifactFinding {
                        path: cache2.to_string_lossy().to_string(),
                        size: total_dir_size(&cache2),
                        risk_level: "medium".into(),
                        description: "Mozilla Firefox browser cache".into(),
                    });
                }
            }
        }
    }
    let edge_cache =
        Path::new(&user_profile).join("AppData\\Local\\Microsoft\\Edge\\User Data\\Default\\Cache");
    if edge_cache.exists() {
        findings.push(ArtifactFinding {
            path: edge_cache.to_string_lossy().to_string(),
            size: total_dir_size(&edge_cache),
            risk_level: "medium".into(),
            description: "Microsoft Edge browser cache".into(),
        });
    }
    Ok(findings)
}

pub fn scan_thumbnail_databases() -> Result<Vec<ArtifactFinding>> {
    let mut findings = Vec::new();
    let user_profile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".into());
    let thumb_dir = Path::new(&user_profile).join("AppData\\Local\\Microsoft\\Windows\\Explorer");
    if thumb_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&thumb_dir) {
            for entry in entries.flatten() {
                let name = entry
                    .path()
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if name.starts_with("thumbcache_") || name == "IconCache.db" {
                    if let Ok(meta) = entry.metadata() {
                        findings.push(ArtifactFinding {
                            path: entry.path().to_string_lossy().to_string(),
                            size: meta.len(),
                            risk_level: "medium".into(),
                            description: format!("Thumbnail database: {}", name),
                        });
                    }
                }
            }
        }
    }
    Ok(findings)
}

pub fn scan_temp_files() -> Result<Vec<ArtifactFinding>> {
    let mut findings = Vec::new();
    let temp = std::env::var("TEMP")
        .or_else(|_| std::env::var("TMP"))
        .unwrap_or_else(|_| "C:\\Windows\\Temp".into());
    let temp_path = Path::new(&temp);
    if temp_path.exists() {
        let size = total_dir_size(temp_path);
        if size > 0 {
            findings.push(ArtifactFinding {
                path: temp_path.to_string_lossy().to_string(),
                size,
                risk_level: "low".into(),
                description: format!(
                    "User temp directory (size: {})",
                    crate::utils::file_size_human(size)
                ),
            });
        }
    }
    let sys_temp = Path::new("C:\\Windows\\Temp");
    if sys_temp.exists() {
        let size = total_dir_size(sys_temp);
        if size > 0 {
            findings.push(ArtifactFinding {
                path: sys_temp.to_string_lossy().to_string(),
                size,
                risk_level: "low".into(),
                description: format!(
                    "System temp directory (size: {})",
                    crate::utils::file_size_human(size)
                ),
            });
        }
    }
    Ok(findings)
}

pub fn scan_recent_files() -> Result<Vec<ArtifactFinding>> {
    let mut findings = Vec::new();
    let appdata =
        std::env::var("APPDATA").unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Roaming".into());
    let recent_path = Path::new(&appdata).join("Microsoft\\Windows\\Recent");
    if recent_path.exists() {
        if let Ok(entries) = std::fs::read_dir(&recent_path) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if let Some(lnk_name) = entry.path().file_name() {
                        findings.push(ArtifactFinding {
                            path: entry.path().to_string_lossy().to_string(),
                            size: meta.len(),
                            risk_level: "medium".into(),
                            description: format!(
                                "Recent file shortcut: {}",
                                lnk_name.to_string_lossy()
                            ),
                        });
                    }
                }
            }
        }
    }
    Ok(findings)
}

pub fn scan_pagefile() -> Result<Vec<ArtifactFinding>> {
    let mut findings = Vec::new();
    let pagefile = Path::new("C:\\pagefile.sys");
    if pagefile.exists() {
        if let Ok(meta) = pagefile.metadata() {
            findings.push(ArtifactFinding {
                path: pagefile.to_string_lossy().to_string(),
                size: meta.len(),
                risk_level: "high".into(),
                description: format!(
                    "Windows pagefile (size: {})",
                    crate::utils::file_size_human(meta.len())
                ),
            });
        }
    }
    Ok(findings)
}

pub fn scan_hibernation() -> Result<Vec<ArtifactFinding>> {
    let mut findings = Vec::new();
    let hiberfile = Path::new("C:\\hiberfil.sys");
    if hiberfile.exists() {
        if let Ok(meta) = hiberfile.metadata() {
            findings.push(ArtifactFinding {
                path: hiberfile.to_string_lossy().to_string(),
                size: meta.len(),
                risk_level: "high".into(),
                description: format!(
                    "Windows hibernation file (size: {})",
                    crate::utils::file_size_human(meta.len())
                ),
            });
        }
    }
    Ok(findings)
}

pub fn scan_recycle_bin() -> Result<Vec<ArtifactFinding>> {
    let mut findings = Vec::new();
    let recycle = Path::new("C:\\$Recycle.Bin");
    if recycle.exists() {
        if let Ok(entries) = std::fs::read_dir(recycle) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let size = total_dir_size(&path);
                    let sid = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    findings.push(ArtifactFinding {
                        path: path.to_string_lossy().to_string(),
                        size,
                        risk_level: "high".into(),
                        description: format!(
                            "Recycle Bin for user SID: {} (size: {})",
                            sid,
                            crate::utils::file_size_human(size)
                        ),
                    });
                }
            }
        }
    }
    Ok(findings)
}

fn total_dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total += meta.len();
                } else if meta.is_dir() {
                    total += total_dir_size(&entry.path());
                }
            }
        }
    }
    total
}
