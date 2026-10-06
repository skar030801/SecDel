use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationResult {
    pub path: String,
    pub deleted: bool,
    pub bytes_remaining: u64,
    pub random_data_confirmed: bool,
    pub confidence: f64,
}

pub fn verify_file_deletion(path: &Path) -> Result<VerificationResult> {
    let path_str = path.to_string_lossy().to_string();

    if !path.exists() {
        return Ok(VerificationResult {
            path: path_str,
            deleted: true,
            bytes_remaining: 0,
            random_data_confirmed: false,
            confidence: 99.0,
        });
    }

    let metadata = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => {
            return Ok(VerificationResult {
                path: path_str,
                deleted: true,
                bytes_remaining: 0,
                random_data_confirmed: false,
                confidence: 95.0,
            });
        }
    };

    let file_len = metadata.len();

    let remaining_data = file_len;

    if file_len == 0 {
        return Ok(VerificationResult {
            path: path_str,
            deleted: false,
            bytes_remaining: 0,
            random_data_confirmed: false,
            confidence: 50.0,
        });
    }

    let random_data_confirmed = check_random_data(path);

    let confidence = if remaining_data == 0 {
        if random_data_confirmed {
            95.0
        } else {
            80.0
        }
    } else if random_data_confirmed {
        70.0
    } else if remaining_data < 1024 {
        50.0
    } else {
        30.0
    };

    Ok(VerificationResult {
        path: path_str,
        deleted: false,
        bytes_remaining: remaining_data,
        random_data_confirmed,
        confidence,
    })
}

fn check_random_data(path: &Path) -> bool {
    match std::fs::read(path) {
        Ok(data) => {
            if data.is_empty() {
                return false;
            }

            let sample_size = data.len().min(4096);
            let sample = &data[..sample_size];

            let all_same = sample.iter().all(|&b| b == sample[0]);
            if all_same {
                return false;
            }

            let entropy = compute_entropy(sample);
            entropy > 7.0
        }
        Err(_) => false,
    }
}

fn compute_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut freq = [0u64; 256];
    for &byte in data {
        freq[byte as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0f64;
    for &count in freq.iter() {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    entropy
}
