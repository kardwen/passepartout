use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("pass error: {0}")]
    Pass(String),

    #[error("gpgme error: {0}")]
    Gpgme(#[from] gpgme::Error),

    #[error("Clipboard error: {0}")]
    Clipboard(#[from] arboard::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("InvalidUtf8 error: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("OTP error: {0}")]
    Otp(#[from] totp_rs::TotpUrlError),
}
