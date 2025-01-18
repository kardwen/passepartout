//! 🔑 Library for pass
//!
//! # Features
//!
//! - `gpgme`: Decryption with `gpgme`, `GnuPG` implementation compatible with `pass` (default)
//! - `sequoia`: Decryption with `sequoia-openpgp`, `OpenPGP` implementation (experimental)

mod clipboard;
mod error;
mod pass;

pub use error::Error;
pub use pass::{
    copy_id, copy_line, copy_login, copy_otp, copy_password, decrypt_password_file, generate_otp,
    PasswordInfo, PasswordStore,
};
