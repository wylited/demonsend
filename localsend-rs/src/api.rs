use std::{collections::HashMap, time::SystemTime};

use uuid::Uuid;
use warp::{filters::body::bytes, reject::Rejection, reply::Reply};

use crate::{client::{Client, SessionStatus, TransferSession},  PrepareUploadRequest, PrepareUploadResponse};

#[derive(Debug)]
pub enum ApiError {
    InvalidParameters,
    InvalidToken,
    SessionNotFound,
    UnknownError(String),
}

impl warp::reject::Reject for ApiError {}

impl Client {
    pub async fn handle_prepare_upload(&self, request: PrepareUploadRequest) -> Result<impl Reply, Rejection> {
        let session_id = Uuid::new_v4().to_string();
        let mut file_tokens = HashMap::new();

        // TODO implement notification and rejection

        for (file_id, _) in &request.files {
            file_tokens.insert(file_id.clone(), Uuid::new_v4().to_string());
        }

        let session = TransferSession {
            session_id: session_id.clone(),
            device_info: request.device_info,
            files: request.files,
            file_tokens: file_tokens.clone(),
            created_at: SystemTime::now(),
            status: SessionStatus::Preparing,
        };

        self.sessions.lock().await.insert(session_id.clone(), session);

        let response = PrepareUploadResponse {
            session_id,
            file_tokens,
        };

        Ok(warp::reply::json(&response))
    }

    pub async fn handle_upload(
        &self,
        session_id: &str,
        file_id: &str,
        token: &str,
        bytes: bytes::Bytes,
    ) -> Result<impl Reply, Rejection> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| warp::reject::custom(ApiError::SessionNotFound))?;

        // Validate token
        if session.file_tokens.get(file_id) != Some(&token.to_string()) {
            return Err(warp::reject::custom(ApiError::InvalidToken));
        }

        // Get file metadata
        let file_metadata = session
            .files
            .get(file_id)
            .ok_or_else(|| warp::reject::custom(ApiError::UnknownError("File not found".to_string())))?;

        // Create directory if it doesn't exist
        tokio::fs::create_dir_all(&*self.download_dir)
            .await
            .map_err(|e| warp::reject::custom(ApiError::UnknownError(e.to_string())))?;

        // Create file path
        let file_path = self.download_dir.join(&file_metadata.file_name);

        // Write file
        tokio::fs::write(file_path, bytes)
            .await
            .map_err(|e| warp::reject::custom(ApiError::UnknownError(e.to_string())))?;

        session.status = SessionStatus::Transferring;

        Ok(warp::reply())
}

    pub async fn handle_cancel(&self, session_id: &str) -> Result<impl Reply, Rejection> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = SessionStatus::Cancelled;
            sessions.remove(session_id);
        }
        Ok(warp::reply())
    }
}

pub fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Rejection> {
    if let Some(ApiError::InvalidParameters) = err.find() {
        Ok(warp::reply::with_status(
            "Missing parameters",
            warp::http::StatusCode::BAD_REQUEST,
        ))
    } else if let Some(ApiError::InvalidToken) = err.find() {
        Ok(warp::reply::with_status(
            "Invalid token or IP address",
            warp::http::StatusCode::FORBIDDEN,
        ))
    } else if let Some(ApiError::SessionNotFound) = err.find() {
        Ok(warp::reply::with_status(
            "Session not found",
            warp::http::StatusCode::NOT_FOUND,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Internal server error",
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}
