use crate::error::Result;
use crate::scanner::scanner;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub path: String,
    pub finding_type: String,
    pub size: u64,
    pub risk_level: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub alternate_data_streams: Vec<Finding>,
    pub shadow_copies: Vec<Finding>,
    pub restore_points: Vec<Finding>,
    pub browser_caches: Vec<Finding>,
    pub thumbnail_databases: Vec<Finding>,
    pub temp_files: Vec<Finding>,
    pub recent_file_records: Vec<Finding>,
    pub pagefile_artifacts: Vec<Finding>,
    pub hibernation_artifacts: Vec<Finding>,
    pub recycle_bin: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub current: u64,
    pub total: u64,
    pub phase: String,
}

#[tauri::command]
pub async fn scan_artifacts(
    app_handle: AppHandle,
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<ScanResult> {
    let total_phases: u64 = 10;

    app_handle
        .emit(
            "scan_progress",
            ScanProgress {
                current: 0,
                total: total_phases,
                phase: "scanning_ads".into(),
            },
        )
        .ok();

    let ads_findings = if !paths.is_empty() {
        let mut findings = Vec::new();
        for p in &paths {
            if state.cancelled.load(Ordering::Relaxed) {
                state.cancelled.store(false, Ordering::Relaxed);
                return Err(crate::error::SecDelError::OperationCancelled);
            }
            findings.extend(scanner::scan_alternate_data_streams(Path::new(p)));
        }
        findings
    } else {
        Vec::new()
    };

    if state.cancelled.load(Ordering::Relaxed) {
        state.cancelled.store(false, Ordering::Relaxed);
        return Err(crate::error::SecDelError::OperationCancelled);
    }

    app_handle
        .emit(
            "scan_progress",
            ScanProgress {
                current: 1,
                total: total_phases,
                phase: "scanning_shadow_copies".into(),
            },
        )
        .ok();

    let shadow_copies = scanner::scan_shadow_copies()?;

    if state.cancelled.load(Ordering::Relaxed) {
        state.cancelled.store(false, Ordering::Relaxed);
        return Err(crate::error::SecDelError::OperationCancelled);
    }

    app_handle
        .emit(
            "scan_progress",
            ScanProgress {
                current: 2,
                total: total_phases,
                phase: "scanning_restore_points".into(),
            },
        )
        .ok();

    let restore_points = scanner::scan_restore_points()?;

    if state.cancelled.load(Ordering::Relaxed) {
        state.cancelled.store(false, Ordering::Relaxed);
        return Err(crate::error::SecDelError::OperationCancelled);
    }

    app_handle
        .emit(
            "scan_progress",
            ScanProgress {
                current: 3,
                total: total_phases,
                phase: "scanning_browser_caches".into(),
            },
        )
        .ok();

    let browser_caches = scanner::scan_browser_caches()?;

    if state.cancelled.load(Ordering::Relaxed) {
        state.cancelled.store(false, Ordering::Relaxed);
        return Err(crate::error::SecDelError::OperationCancelled);
    }

    app_handle
        .emit(
            "scan_progress",
            ScanProgress {
                current: 4,
                total: total_phases,
                phase: "scanning_thumbnails".into(),
            },
        )
        .ok();

    let thumbnail_databases = scanner::scan_thumbnail_databases()?;

    if state.cancelled.load(Ordering::Relaxed) {
        state.cancelled.store(false, Ordering::Relaxed);
        return Err(crate::error::SecDelError::OperationCancelled);
    }

    app_handle
        .emit(
            "scan_progress",
            ScanProgress {
                current: 5,
                total: total_phases,
                phase: "scanning_temp_files".into(),
            },
        )
        .ok();

    let temp_files = scanner::scan_temp_files()?;

    if state.cancelled.load(Ordering::Relaxed) {
        state.cancelled.store(false, Ordering::Relaxed);
        return Err(crate::error::SecDelError::OperationCancelled);
    }

    app_handle
        .emit(
            "scan_progress",
            ScanProgress {
                current: 6,
                total: total_phases,
                phase: "scanning_recent_files".into(),
            },
        )
        .ok();

    let recent_file_records = scanner::scan_recent_files()?;

    if state.cancelled.load(Ordering::Relaxed) {
        state.cancelled.store(false, Ordering::Relaxed);
        return Err(crate::error::SecDelError::OperationCancelled);
    }

    app_handle
        .emit(
            "scan_progress",
            ScanProgress {
                current: 7,
                total: total_phases,
                phase: "scanning_pagefile".into(),
            },
        )
        .ok();

    let pagefile_artifacts = scanner::scan_pagefile()?;

    if state.cancelled.load(Ordering::Relaxed) {
        state.cancelled.store(false, Ordering::Relaxed);
        return Err(crate::error::SecDelError::OperationCancelled);
    }

    app_handle
        .emit(
            "scan_progress",
            ScanProgress {
                current: 8,
                total: total_phases,
                phase: "scanning_hibernation".into(),
            },
        )
        .ok();

    let hibernation_artifacts = scanner::scan_hibernation()?;

    if state.cancelled.load(Ordering::Relaxed) {
        state.cancelled.store(false, Ordering::Relaxed);
        return Err(crate::error::SecDelError::OperationCancelled);
    }

    app_handle
        .emit(
            "scan_progress",
            ScanProgress {
                current: 9,
                total: total_phases,
                phase: "scanning_recycle_bin".into(),
            },
        )
        .ok();

    let recycle_bin = scanner::scan_recycle_bin()?;

    app_handle
        .emit(
            "scan_progress",
            ScanProgress {
                current: 10,
                total: total_phases,
                phase: "complete".into(),
            },
        )
        .ok();

    Ok(ScanResult {
        alternate_data_streams: ads_findings
            .into_iter()
            .map(|f| Finding {
                path: f.path,
                finding_type: "alternate_data_stream".into(),
                size: f.size,
                risk_level: f.risk_level,
                description: f.description,
            })
            .collect(),
        shadow_copies: shadow_copies
            .into_iter()
            .map(|f| Finding {
                path: f.path,
                finding_type: "shadow_copy".into(),
                size: f.size,
                risk_level: f.risk_level,
                description: f.description,
            })
            .collect(),
        restore_points: restore_points
            .into_iter()
            .map(|f| Finding {
                path: f.path,
                finding_type: "restore_point".into(),
                size: f.size,
                risk_level: f.risk_level,
                description: f.description,
            })
            .collect(),
        browser_caches: browser_caches
            .into_iter()
            .map(|f| Finding {
                path: f.path,
                finding_type: "browser_cache".into(),
                size: f.size,
                risk_level: f.risk_level,
                description: f.description,
            })
            .collect(),
        thumbnail_databases: thumbnail_databases
            .into_iter()
            .map(|f| Finding {
                path: f.path,
                finding_type: "thumbnail_database".into(),
                size: f.size,
                risk_level: f.risk_level,
                description: f.description,
            })
            .collect(),
        temp_files: temp_files
            .into_iter()
            .map(|f| Finding {
                path: f.path,
                finding_type: "temp_file".into(),
                size: f.size,
                risk_level: f.risk_level,
                description: f.description,
            })
            .collect(),
        recent_file_records: recent_file_records
            .into_iter()
            .map(|f| Finding {
                path: f.path,
                finding_type: "recent_file_record".into(),
                size: f.size,
                risk_level: f.risk_level,
                description: f.description,
            })
            .collect(),
        pagefile_artifacts: pagefile_artifacts
            .into_iter()
            .map(|f| Finding {
                path: f.path,
                finding_type: "pagefile_artifact".into(),
                size: f.size,
                risk_level: f.risk_level,
                description: f.description,
            })
            .collect(),
        hibernation_artifacts: hibernation_artifacts
            .into_iter()
            .map(|f| Finding {
                path: f.path,
                finding_type: "hibernation_artifact".into(),
                size: f.size,
                risk_level: f.risk_level,
                description: f.description,
            })
            .collect(),
        recycle_bin: recycle_bin
            .into_iter()
            .map(|f| Finding {
                path: f.path,
                finding_type: "recycle_bin".into(),
                size: f.size,
                risk_level: f.risk_level,
                description: f.description,
            })
            .collect(),
    })
}
