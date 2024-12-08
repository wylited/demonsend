// download.rs
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    http::{header, StatusCode},
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::protocol::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub token: String,
}

pub async fn handle_download(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, (StatusCode, String)> {
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

    let file_path = state.download_dir.join(&file_metadata.fileName);

    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    let mut file = File::open(&file_path).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut contents = Vec::new();
    file.read_to_end(&mut contents).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, file_metadata.fileType.as_str())
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file_metadata.fileName))
        .header(header::CONTENT_LENGTH, contents.len())
        .body(contents.into())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(response)
}

pub async fn handle_cancel(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<(), (StatusCode, String)> {
    let session_id = params.get("sessionId")
        .ok_or((StatusCode::BAD_REQUEST, "Missing sessionId".to_string()))?;

    let mut sessions = state.active_sessions.lock().await;
    sessions.remove(session_id);

    Ok(())
}
