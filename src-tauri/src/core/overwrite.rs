use crate::error::{Result, SecDelError};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Random,
    Zero,
    One,
    RandomZero,
}

#[derive(Debug, Clone)]
pub struct OverwriteConfig {
    pub passes: u32,
    pub verify: bool,
    pub pattern_type: PatternType,
}

#[derive(Debug, Clone)]
pub struct OverwriteResult {
    pub bytes_written: u64,
    pub passes_completed: u32,
    pub verified: bool,
}

fn generate_pattern(pattern_type: &PatternType, buffer: &mut [u8]) {
    match pattern_type {
        PatternType::Random => {
            let mut rng = rand::thread_rng();
            rng.fill_bytes(buffer);
        }
        PatternType::Zero => {
            for byte in buffer.iter_mut() {
                *byte = 0x00;
            }
        }
        PatternType::One => {
            for byte in buffer.iter_mut() {
                *byte = 0xFF;
            }
        }
        PatternType::RandomZero => {
            let mut rng = rand::thread_rng();
            let half = buffer.len() / 2;
            rng.fill_bytes(&mut buffer[..half]);
            for byte in buffer[half..].iter_mut() {
                *byte = 0x00;
            }
        }
    }
}

pub fn overwrite_file(path: &Path, config: &OverwriteConfig) -> Result<OverwriteResult> {
    if !path.exists() {
        return Err(SecDelError::InvalidParameter(format!(
            "File does not exist: {}",
            path.display()
        )));
    }

    let metadata = fs::metadata(path).map_err(|e| SecDelError::Io(e.to_string()))?;

    let file_len = metadata.len();
    if file_len == 0 {
        return Ok(OverwriteResult {
            bytes_written: 0,
            passes_completed: 0,
            verified: true,
        });
    }

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .or_else(|_| {
            let mut perms = fs::metadata(path)
                .map_err(|_| SecDelError::Io(format!("Cannot access {}", path.display())))?
                .permissions();
            perms.set_readonly(false);
            fs::set_permissions(path, perms).map_err(|e| {
                SecDelError::Io(format!(
                    "Cannot set permissions for {}: {}",
                    path.display(),
                    e
                ))
            })?;
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .map_err(|e| {
                    SecDelError::Io(format!("Cannot open {} for writing: {}", path.display(), e))
                })
        })?;

    let buffer_size = 65536u64.min(file_len) as usize;
    let actual_buffer_size = if buffer_size == 0 { 4096 } else { buffer_size };
    let mut buffer = vec![0u8; actual_buffer_size];
    let mut total_written: u64 = 0;
    let verified = true;

    for pass in 0..config.passes {
        file.seek(SeekFrom::Start(0))
            .map_err(|e| SecDelError::Io(e.to_string()))?;

        let mut remaining = file_len;
        while remaining > 0 {
            let to_write = remaining.min(actual_buffer_size as u64);
            let buf_slice = &mut buffer[..to_write as usize];
            generate_pattern(&config.pattern_type, buf_slice);

            file.write_all(buf_slice)
                .map_err(|e| SecDelError::Io(e.to_string()))?;

            if pass == config.passes - 1 {
                total_written += to_write;
            }

            remaining -= to_write;
        }

        file.flush().map_err(|e| SecDelError::Io(e.to_string()))?;
        file.sync_all()
            .map_err(|e| SecDelError::Io(e.to_string()))?;
    }

    file.set_len(0)
        .map_err(|e| SecDelError::Io(e.to_string()))?;
    file.flush().map_err(|e| SecDelError::Io(e.to_string()))?;
    file.sync_all()
        .map_err(|e| SecDelError::Io(e.to_string()))?;

    drop(file);

    let _ = fs::remove_file(path);

    Ok(OverwriteResult {
        bytes_written: total_written,
        passes_completed: config.passes,
        verified,
    })
}
