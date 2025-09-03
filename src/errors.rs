#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Processing error: {0}")]
    ProcessError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
    #[error("Unknown trade pair: {0}")]
    UnknownTrade(String),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    ParseLevelError(#[from] log::ParseLevelError),
    #[error(transparent)]
    SetLoggerError(#[from] log::SetLoggerError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

pub type AppResult<T> = Result<T, AppError>;
