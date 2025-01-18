use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("pass error: {0}")]
    Pass(String),

    #[cfg(feature = "gpgme")]
    #[error("gpgme error: {0}")]
    Gpgme(#[from] gpgme::Error),

    #[cfg(feature = "sequoia")]
    #[error("sequoia-openpgp error: {0}")]
    Sequoia(#[from] sequoia_openpgp::Error),

    #[cfg(feature = "sequoia")]
    #[error("sequoia-gpg-agent error: {0}")]
    SequoiaAgent(#[from] sequoia_gpg_agent::Error),

    #[error("Clipboard error: {0}")]
    Clipboard(#[from] arboard::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("InvalidUtf8 error: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("OTP error: {0}")]
    Otp(#[from] totp_rs::TotpUrlError),
}
