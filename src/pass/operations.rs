use gpgme::{Context, Protocol};
use std::{cell::RefCell, path::Path, time};
use totp_rs::TOTP;

use crate::{clipboard::copy_to_clipboard, Error};

thread_local! {
    static GPG_CONTEXT: RefCell<Option<Context>> = const {RefCell::new(None)};
}

/// Retrieves the contents of a password file.
///
/// This operation is synchronous and will block until GPG decryption completes.
pub fn decrypt_password_file(file_path: &Path) -> Result<String, Error> {
    GPG_CONTEXT.with(|ctx| {
        let mut ctx_ref = ctx.borrow_mut();
        if ctx_ref.is_none() {
            *ctx_ref = Some(Context::from_protocol(Protocol::OpenPgp)?);
        }

        let ctx = ctx_ref.as_mut().unwrap();
        let mut cipher = std::fs::read(file_path)?;
        let mut plain = Vec::new();

        ctx.decrypt(&mut cipher, &mut plain)?;

        String::from_utf8(plain).map_err(|e| e.into())
    })
}

/// Copies the password ID to the system clipboard.
pub fn copy_id(pass_id: String) -> Result<(), Error> {
    copy_to_clipboard(&pass_id, false)
}

/// Copies the password from a file to the system clipboard, will be cleared after 45 seconds.
///
/// This operation is synchronous.
pub fn copy_password(file_path: &Path) -> Result<(), Error> {
    // Decrypt file and extract password on first line
    let file_contents = decrypt_password_file(file_path)?;
    let password = file_contents
        .lines()
        .next()
        .ok_or_else(|| Error::Pass("no password found".to_string()))?;

    copy_to_clipboard(password, true)
}

/// Copies the login from a file to the system clipboard, will be cleared after some time.
///
/// This operation is synchronous.
pub fn copy_login(file_path: &Path) -> Result<(), Error> {
    // Decrypt file and extract login on second line
    let file_contents = decrypt_password_file(file_path)?;
    let login = file_contents
        .lines()
        .nth(1)
        .ok_or_else(|| Error::Pass("no login found".to_string()))?;

    copy_to_clipboard(login, true)
}

/// Generates and returns a one-time password (OTP).
///
/// This operation is synchronous.
pub fn generate_otp(file_path: &Path) -> Result<String, Error> {
    // Decrypt file and find line starting with otpauth://
    let file_contents = decrypt_password_file(file_path)?;

    // Find otpauth:// in the file contents
    let otpauth = file_contents
        .lines()
        .find(|line| line.starts_with("otpauth://"))
        .ok_or_else(|| Error::Pass("no OTP URL found".to_string()))?;

    let totp = TOTP::from_url(otpauth)?;

    totp.generate_current()
        .map_err(|e: time::SystemTimeError| Error::Pass(format!("failed to generate OTP: {}", e)))
}

/// Generates a one-time password (OTP) and copies it to the system clipboard.
///
/// This operation is synchronous.
pub fn copy_otp(file_path: &Path) -> Result<(), Error> {
    let otp = generate_otp(file_path)?;
    copy_to_clipboard(&otp, false)
}
