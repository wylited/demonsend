#[derive(Debug, thiserror::Error)]
pub enum LocalSendError {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid PIN")]
    InvalidPin,

    #[error("Session blocked")]
    SessionBlocked,

    #[error("Too many requests")]
    TooManyRequests,

    #[error("Not a file")]
    NotAFile,
}

pub type Result<T> = std::result::Result<T, LocalSendError>;
