use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
pub enum SecDelError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Operation cancelled")]
    OperationCancelled,

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("TRIM operation failed: {0}")]
    TrimFailed(String),

    #[error("NVMe operation failed: {0}")]
    NvmeFailed(String),

    #[error("ATA operation failed: {0}")]
    AtaFailed(String),

    #[error("Secure erase failed: {0}")]
    SecureEraseFailed(String),

    #[error("Crypto erase failed: {0}")]
    CryptoEraseFailed(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("WMI query failed: {0}")]
    WmiFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Report generation failed: {0}")]
    ReportFailed(String),

    #[error("Admin privileges required: {0}")]
    AdminRequired(String),

    #[error("{0}")]
    Other(String),
}

impl From<std::io::Error> for SecDelError {
    fn from(e: std::io::Error) -> Self {
        SecDelError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for SecDelError {
    fn from(e: serde_json::Error) -> Self {
        SecDelError::Serialization(e.to_string())
    }
}

impl From<windows::core::Error> for SecDelError {
    fn from(e: windows::core::Error) -> Self {
        SecDelError::Other(format!("Windows error: {}", e))
    }
}

pub type Result<T> = std::result::Result<T, SecDelError>;
