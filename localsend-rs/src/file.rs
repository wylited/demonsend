use serde::{Deserialize, Serialize};
use sha256::digest;
use std::fs;
use std::path::Path;
use uuid::Uuid;

use crate::LocalSendError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    pub file_name: String,
    pub size: u64,
    pub file_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<FileMetadataExt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadataExt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessed: Option<String>,
}

impl FileMetadata {
    pub fn new(
        id: String,
        file_name: String,
        size: u64,
        file_type: String,
        sha256: Option<String>,
        preview: Option<String>,
        metadata: Option<FileMetadataExt>,
    ) -> Self {
        Self {
            id,
            file_name,
            size,
            file_type,
            sha256,
            preview,
            metadata,
        }
    }

    pub fn from_path(path: &Path) -> Result<Self, LocalSendError> {
        let id = Uuid::new_v4().to_string();

        let file_name = path
            .file_name()
            .ok_or_else(|| LocalSendError::Unknown("Failed to get filename".to_string()))?
            .to_str()
            .ok_or_else(|| {
                LocalSendError::Unknown("Failed to convert filename to string".to_string())
            })?
            .to_string();

        let size = path.metadata().map_err(LocalSendError::Io)?.len();

        let file_type = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(String::from)
            .unwrap_or_else(|| "".to_string());

        let bytes = fs::read(path).map_err(LocalSendError::Io)?;

        let sha = Some(digest(bytes).to_string());

        Ok(Self::new(id, file_name, size, file_type, sha, None, None))
    }
}
