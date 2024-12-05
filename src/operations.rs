use arboard::Clipboard;
use std::{
    ffi::OsStr,
    process::{Command, Stdio},
    sync::Mutex,
};

use crate::{error::PasswordError, events::PasswordEvent};

static CLIPBOARD: Mutex<Option<Clipboard>> = Mutex::new(None);

/// Copies the password ID to the system clipboard.
pub fn copy_id(pass_id: String) -> Result<(), PasswordError> {
    let mut clipboard = CLIPBOARD
        .lock()
        .expect("another thread holding the lock paniced");

    if clipboard.is_none() {
        *clipboard = Clipboard::new().ok();
    }

    match clipboard.as_mut() {
        Some(clipboard) => match clipboard.set_text(pass_id) {
            Ok(()) => Ok(()),
            Err(e) => Err(PasswordError::ClipboardError(e)),
        },
        None => Err(PasswordError::ClipboardUnavailable),
    }
}

/// Copies the password to the system clipboard.
///
/// This operation is synchronous and will block until the `pass` command completes.
pub fn copy_password(pass_id: String) -> Result<PasswordEvent, PasswordError> {
    let status = Command::new("pass")
        .arg(OsStr::new(&pass_id))
        .arg("--clip")
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .expect("failed to execute process");
    if status.success() {
        let message = "Password copied to clipboard, clears after 45 seconds".to_string();
        Ok(PasswordEvent::Status(Ok(Some(message))))
    } else {
        Err(PasswordError::PassError(status.to_string()))
    }
}

/// Copies the login to the system clipboard.
///
/// This operation is synchronous and will block until the `pass` command completes.
pub fn copy_login(pass_id: String) -> Result<PasswordEvent, PasswordError> {
    let status = Command::new("pass")
        .arg(OsStr::new(&pass_id))
        .arg("--clip=2")
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .expect("failed to execute process");
    if status.success() {
        let message = "Login copied to clipboard, clears after 45 seconds".to_string();
        Ok(PasswordEvent::Status(Ok(Some(message))))
    } else {
        Err(PasswordError::PassError(status.to_string()))
    }
}

/// Copies the one-time password (OTP) to the system clipboard.
///
/// This operation is synchronous and will block until the `pass` command completes.
pub fn copy_otp(pass_id: String) -> Result<PasswordEvent, PasswordError> {
    let status = Command::new("pass")
        .arg("otp")
        .arg("code")
        .arg(OsStr::new(&pass_id))
        .arg("--clip")
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .expect("failed to execute process");
    if status.success() {
        let message = "One-time password copied to clipboard".to_string();
        Ok(PasswordEvent::Status(Ok(Some(message))))
    } else {
        Err(PasswordError::PassError(status.to_string()))
    }
}

/// Retrieves the one-time password (OTP).
///
/// This operation is synchronous and will block until the `pass` command completes.
pub fn fetch_otp(pass_id: String) -> Result<PasswordEvent, PasswordError> {
    let output = Command::new("pass")
        .arg("otp")
        .arg("code")
        .arg(OsStr::new(&pass_id))
        .output()
        .expect("failed to execute process");
    if output.status.success() {
        let one_time_password = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(PasswordEvent::OneTimePassword {
            pass_id,
            one_time_password,
        })
    } else {
        let message = String::from_utf8_lossy(&output.stderr).to_string();
        Err(PasswordError::PassError(message))
    }
}

/// Retrieves the contents of a password file.
///
/// This operation is synchronous and will block until the `pass` command completes.
pub fn fetch_entry(pass_id: String) -> Result<PasswordEvent, PasswordError> {
    let output = Command::new("pass")
        .arg(OsStr::new(&pass_id))
        .output()
        .expect("failed to execute process");
    if output.status.success() {
        let file_contents = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(PasswordEvent::PasswordInfo {
            pass_id,
            file_contents,
        })
    } else {
        let message = String::from_utf8_lossy(&output.stderr).to_string();
        Err(PasswordError::PassError(message))
    }
}
