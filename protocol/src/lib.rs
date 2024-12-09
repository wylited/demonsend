mod client;
mod discovery;
mod file;

use file::FileMetadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Mobile,
    Desktop,
    Web,
    Headless,
    Server,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub alias: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deviceModel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deviceType: Option<DeviceType>,
    pub fingerprint: String,
    pub port: u16,
    pub protocol: String,
    #[serde(default)]
    pub download: bool,
    #[serde(default)]
    pub announce: bool,
}

impl DeviceInfo {
    #[must_use]
    pub fn new(
        alias: String,
        version: String,
        deviceModel: Option<String>,
        deviceType: Option<DeviceType>,
        port: u16,
        protocol: String,
        download: bool,
        announce: bool,
    ) -> Self {
        Self {
            alias,
            version,
            deviceModel,
            deviceType,
            fingerprint: Uuid::new_v4().to_string(),
            port,
            protocol,
            download,
            announce,
        }
    }

    pub fn default() -> Self {
        Self::new(
            "localsend-rs".to_string(),
            "2.1".to_string(),
            None,
            Some(DeviceType::Headless),
            53317,
            "http".to_string(),
            true,
            true,
        )
    }
}

// Error handling
#[derive(Debug, thiserror::Error)]
pub enum LocalSendError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid PIN")]
    InvalidPin,
    #[error("PIN required")]
    PinRequired,
    #[error("Session blocked")]
    SessionBlocked,
    #[error("Too many requests")]
    TooManyRequests,
    #[error("Port Bound")]
    PortBound,
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, LocalSendError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareUploadRequest {
    pub device_info: DeviceInfo,
    pub files: HashMap<String, FileMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareUploadResponse {
    pub session_id: String,
    pub file_tokens: HashMap<String, String>, // file_id -> token
}
