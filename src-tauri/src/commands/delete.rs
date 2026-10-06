use crate::core::overwrite::{overwrite_file, OverwriteConfig, PatternType};
use crate::core::trim::{execute_trim, TrimResult};
use crate::core::verify::verify_file_deletion;
use crate::error::{Result, SecDelError};
use crate::utils;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::Instant;
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationResult {
    pub path: String,
    pub deleted: bool,
    pub bytes_remaining: u64,
    pub random_data_confirmed: bool,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletionReport {
    pub files_processed: Vec<String>,
    pub bytes_overwritten: u64,
    pub passes_completed: u32,
    pub trim_operations: Vec<TrimResult>,
    pub verification_results: Vec<VerificationResult>,
    pub start_time: String,
    pub end_time: String,
    pub confidence_score: f64,
    pub level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressPayload {
    pub current: u64,
    pub total: u64,
    pub phase: String,
    pub file: String,
}

fn get_pattern_for_level(level: u8, pass: u32) -> PatternType {
    match level {
        1 => PatternType::Random,
        2 => PatternType::Random,
        3 => match pass % 3 {
            0 => PatternType::Random,
            1 => PatternType::One,
            _ => PatternType::Random,
        },
        _ => match pass % 7 {
            0 => PatternType::Random,
            1 => PatternType::Zero,
            2 => PatternType::One,
            3 => PatternType::Random,
            4 => PatternType::Zero,
            5 => PatternType::One,
            _ => PatternType::RandomZero,
        },
    }
}

fn get_passes_for_level(level: u8) -> u32 {
    match level {
        1 => 1,
        2 => 1,
        3 => 3,
        _ => 7,
    }
}

#[tauri::command]
pub async fn delete_files(
    app_handle: AppHandle,
    paths: Vec<String>,
    level: u8,
    state: State<'_, AppState>,
) -> Result<DeletionReport> {
    if level >= 3 && !utils::is_admin() {
        return Err(SecDelError::AdminRequired(
            "Admin privileges required for deletion level >= 3".into(),
        ));
    }

    let total_files = paths.len() as u64;
    let passes = get_passes_for_level(level);
    let start_time = utils::timestamp_iso8601();
    let start_instant = Instant::now();

    let mut files_processed = Vec::new();
    let mut bytes_overwritten: u64 = 0;
    let mut passes_completed: u32 = 0;
    let mut trim_operations = Vec::new();
    let mut verification_results = Vec::new();

    for (idx, file_path) in paths.iter().enumerate() {
        if state.cancelled.load(Ordering::Relaxed) {
            state.cancelled.store(false, Ordering::Relaxed);
            return Err(SecDelError::OperationCancelled);
        }

        let path = Path::new(file_path);
        if !path.exists() {
            continue;
        }

        app_handle
            .emit(
                "progress",
                ProgressPayload {
                    current: idx as u64,
                    total: total_files,
                    phase: "overwriting".into(),
                    file: file_path.clone(),
                },
            )
            .ok();

        for pass in 0..passes {
            if state.cancelled.load(Ordering::Relaxed) {
                state.cancelled.store(false, Ordering::Relaxed);
                return Err(SecDelError::OperationCancelled);
            }
            let pattern = get_pattern_for_level(level, pass);
            let config = OverwriteConfig {
                passes: 1,
                verify: level >= 2,
                pattern_type: pattern,
            };
            match overwrite_file(path, &config) {
                Ok(result) => {
                    bytes_overwritten += result.bytes_written;
                    passes_completed += 1;
                }
                Err(e) => log::error!("Failed to overwrite {}: {}", file_path, e),
            }
        }

        files_processed.push(file_path.clone());

        app_handle
            .emit(
                "progress",
                ProgressPayload {
                    current: idx as u64,
                    total: total_files,
                    phase: "trimming".into(),
                    file: file_path.clone(),
                },
            )
            .ok();

        match execute_trim(path) {
            Ok(trim_op) => trim_operations.push(TrimResult {
                blocks_released: trim_op.blocks_released,
                status: trim_op.status.to_string(),
                estimated_effectiveness: trim_op.estimated_effectiveness,
            }),
            Err(e) => log::warn!("TRIM failed for {}: {}", file_path, e),
        }

        app_handle
            .emit(
                "progress",
                ProgressPayload {
                    current: idx as u64,
                    total: total_files,
                    phase: "verifying".into(),
                    file: file_path.clone(),
                },
            )
            .ok();

        match verify_file_deletion(path) {
            Ok(vr) => verification_results.push(VerificationResult {
                path: vr.path,
                deleted: vr.deleted,
                bytes_remaining: vr.bytes_remaining,
                random_data_confirmed: vr.random_data_confirmed,
                confidence: vr.confidence,
            }),
            Err(e) => {
                log::warn!("Verification failed for {}: {}", file_path, e);
                verification_results.push(VerificationResult {
                    path: file_path.clone(),
                    deleted: !path.exists(),
                    bytes_remaining: 0,
                    random_data_confirmed: false,
                    confidence: 0.0,
                });
            }
        }
    }

    app_handle
        .emit(
            "progress",
            ProgressPayload {
                current: total_files,
                total: total_files,
                phase: "complete".into(),
                file: String::new(),
            },
        )
        .ok();

    let end_time = utils::timestamp_iso8601();
    let elapsed = start_instant.elapsed().as_secs_f64();

    let verified_count = verification_results.iter().filter(|v| v.deleted).count() as f64;
    let total_verified = verification_results.len() as f64;
    let confidence_score = if total_verified > 0.0 {
        let ratio = verified_count / total_verified;
        let level_factor = level as f64 / 7.0;
        let time_factor = (elapsed / 60.0).min(1.0);
        (ratio * 0.5 + level_factor * 0.3 + time_factor * 0.2) * 100.0
    } else {
        0.0
    };

    Ok(DeletionReport {
        files_processed,
        bytes_overwritten,
        passes_completed: passes_completed / passes.max(1),
        trim_operations,
        verification_results,
        start_time,
        end_time,
        confidence_score,
        level,
    })
}

#[tauri::command]
pub fn cancel_operation(state: State<'_, AppState>) {
    state.cancelled.store(true, Ordering::Relaxed);
}

#[tauri::command]
pub async fn verify_deletion(paths: Vec<String>) -> Result<Vec<VerificationResult>> {
    let mut results = Vec::new();
    for path_str in paths {
        let path = Path::new(&path_str);
        match verify_file_deletion(path) {
            Ok(vr) => results.push(VerificationResult {
                path: vr.path,
                deleted: vr.deleted,
                bytes_remaining: vr.bytes_remaining,
                random_data_confirmed: vr.random_data_confirmed,
                confidence: vr.confidence,
            }),
            Err(e) => {
                log::warn!("Verification error for {}: {}", path_str, e);
                results.push(VerificationResult {
                    path: path_str.clone(),
                    deleted: !path.exists(),
                    bytes_remaining: 0,
                    random_data_confirmed: false,
                    confidence: if !path.exists() { 95.0 } else { 0.0 },
                });
            }
        }
    }
    Ok(results)
}
