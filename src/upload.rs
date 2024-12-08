use axum::{
    body::Bytes,
    extract::{Query, State},
    response::Json,
    http::StatusCode,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;
use std::time::SystemTime;

use crate::protocol::{DeviceInfoV2, AppState};

#[derive(Debug)]
pub struct FileTransferSession {
    pub session_id: String,
    pub device_info: DeviceInfoV2,
    pub files: HashMap<String, FileMetadata>,
    pub created_at: SystemTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    pub fileName: String,
    pub size: u64,
    pub fileType: String,
    pub sha256: Option<String>,
    pub preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<FileExtraMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileExtraMetadata {
    pub modified: Option<String>,
    pub accessed: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrepareUploadRequest {
    pub info: DeviceInfoV2,
    pub files: HashMap<String, FileMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrepareUploadResponse {
    pub sessionId: String,
    pub files: HashMap<String, String>, // file_id -> token mapping
}

pub async fn handle_prepare_upload(
    State(state): State<AppState>,
    Json(request): Json<PrepareUploadRequest>,
) -> Result<Json<PrepareUploadResponse>, (StatusCode, String)> {
    let session_id = Uuid::new_v4().to_string();
    let mut file_tokens = HashMap::new();

    for (file_id, _) in &request.files {
        file_tokens.insert(file_id.clone(), Uuid::new_v4().to_string());
    }

    let session = FileTransferSession {
        session_id: session_id.clone(),
        device_info: request.info,
        files: request.files,
        created_at: SystemTime::now(),
    };

    state.active_sessions.lock().await.insert(session_id.clone(), session);

    Ok(Json(PrepareUploadResponse {
        sessionId: session_id,
        files: file_tokens,
    }))
}

pub async fn handle_upload(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    body: Bytes,
) -> Result<(), (StatusCode, String)> {
    let session_id = params.get("sessionId")
        .ok_or((StatusCode::BAD_REQUEST, "Missing sessionId".to_string()))?;
    let file_id = params.get("fileId")
        .ok_or((StatusCode::BAD_REQUEST, "Missing fileId".to_string()))?;
    let token = params.get("token")
        .ok_or((StatusCode::BAD_REQUEST, "Missing token".to_string()))?;

    let sessions = state.active_sessions.lock().await;
    let session = sessions.get(session_id)
        .ok_or((StatusCode::FORBIDDEN, "Invalid session".to_string()))?;

    let file_metadata = session.files.get(file_id)
        .ok_or((StatusCode::BAD_REQUEST, "Invalid fileId".to_string()))?;

    tokio::fs::create_dir_all(&*state.download_dir).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let file_path = state.download_dir.join(&file_metadata.fileName);
    tokio::fs::write(&file_path, body).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    println!("Received file: {}, saved to: {:?}", file_metadata.fileName, file_path);
    Ok(())
}
