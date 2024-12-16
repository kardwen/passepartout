use arboard::Clipboard;
use std::{sync::Mutex, thread, time::Duration};

use crate::Error;

static CLIPBOARD: Mutex<Option<Clipboard>> = Mutex::new(None);
const EXPIRATION_INTERVAL: u64 = 45;

/// Schedules clearing of the clipboard after the specified duration,
/// but only if the clipboard still contains the specified text.
fn schedule_clipboard_clear(text: String, expiry_seconds: u64) {
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(expiry_seconds));

        // Get clipboard
        let mut clipboard = CLIPBOARD
            .lock()
            .expect("another thread holding the lock paniced");

        // Clear clipboard
        if let Some(ref mut clipboard_instance) = *clipboard {
            if let Ok(current_text) = clipboard_instance.get_text() {
                if current_text == text {
                    let _ = clipboard_instance.clear();
                }
            }
        }
    });
}

pub fn copy_to_clipboard(text: &str, expires: bool) -> Result<(), Error> {
    // Get clipboard
    let mut clipboard = CLIPBOARD
        .lock()
        .expect("another thread holding the lock paniced");

    // Initialize clipboard when not already initialized
    if clipboard.is_none() {
        *clipboard = Some(Clipboard::new()?);
    }
    let clipboard_instance = clipboard.as_mut().expect("clipboard should be initialized");

    clipboard_instance.set_text(text)?;
    if expires {
        schedule_clipboard_clear(text.to_string(), EXPIRATION_INTERVAL);
    }

    Ok(())
}
