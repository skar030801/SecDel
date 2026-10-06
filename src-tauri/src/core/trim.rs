use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrimStatus {
    Pending,
    Completed,
    Failed,
    Unsupported,
}

impl std::fmt::Display for TrimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrimStatus::Pending => write!(f, "Pending"),
            TrimStatus::Completed => write!(f, "Completed"),
            TrimStatus::Failed => write!(f, "Failed"),
            TrimStatus::Unsupported => write!(f, "Unsupported"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimOperation {
    pub blocks_released: u64,
    pub status: TrimStatus,
    pub estimated_effectiveness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrimResult {
    pub blocks_released: u64,
    pub status: String,
    pub estimated_effectiveness: f64,
}

pub fn execute_trim(path: &Path) -> Result<TrimOperation> {
    if !path.exists() {
        return Ok(TrimOperation {
            blocks_released: 0,
            status: TrimStatus::Unsupported,
            estimated_effectiveness: 0.0,
        });
    }

    let metadata = std::fs::metadata(path)?;
    let file_len = metadata.len();

    if file_len == 0 {
        return Ok(TrimOperation {
            blocks_released: 0,
            status: TrimStatus::Completed,
            estimated_effectiveness: 100.0,
        });
    }

    let result = crate::platform::trim::trim_file(path);

    match result {
        Ok(blocks) => {
            let effectiveness = if blocks > 0 {
                (blocks as f64 / (file_len / 512).max(1) as f64 * 100.0).min(100.0)
            } else {
                50.0
            };

            Ok(TrimOperation {
                blocks_released: blocks,
                status: TrimStatus::Completed,
                estimated_effectiveness: effectiveness,
            })
        }
        Err(e) => {
            log::warn!(
                "TRIM via platform layer failed ({}), trying sparse fallback",
                e
            );

            let sparse_result = crate::platform::trim::mark_sparse_and_zero(path);

            match sparse_result {
                Ok(blocks) => Ok(TrimOperation {
                    blocks_released: blocks,
                    status: TrimStatus::Completed,
                    estimated_effectiveness: 60.0,
                }),
                Err(_) => Ok(TrimOperation {
                    blocks_released: 0,
                    status: TrimStatus::Unsupported,
                    estimated_effectiveness: 0.0,
                }),
            }
        }
    }
}

pub fn execute_batch_trim(paths: &[&Path]) -> Result<Vec<TrimOperation>> {
    let mut results = Vec::new();
    for path in paths {
        match execute_trim(path) {
            Ok(op) => results.push(op),
            Err(e) => {
                log::warn!("Batch TRIM failed for {}: {}", path.display(), e);
                results.push(TrimOperation {
                    blocks_released: 0,
                    status: TrimStatus::Failed,
                    estimated_effectiveness: 0.0,
                });
            }
        }
    }
    Ok(results)
}

pub fn retrim(path: &Path) -> Result<TrimOperation> {
    execute_trim(path)
}
