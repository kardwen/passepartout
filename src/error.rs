use thiserror::Error;

#[derive(Error, Debug)]
pub enum PasswordError {
    #[error("clipboard not available")]
    ClipboardUnavailable,
    #[error("clipboard error: {0}")]
    ClipboardError(#[from] arboard::Error),
    #[error("pass error: {0}")]
    PassError(String),
}
