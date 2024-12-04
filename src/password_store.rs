use arboard::Clipboard;
use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{mpsc::Sender, Mutex},
    thread::{self, JoinHandle},
};

use crate::{error::PasswordError, events::PasswordEvent, password_info::PasswordInfo};

static CLIPBOARD: Mutex<Option<Clipboard>> = Mutex::new(None);

pub fn get_clipboard() -> &'static Mutex<Option<Clipboard>> {
    let mut clipboard = CLIPBOARD
        .lock()
        .expect("another thread holding the lock paniced");
    if clipboard.is_none() {
        *clipboard = Clipboard::new().ok();
    }
    &CLIPBOARD
}

pub fn copy_id(pass_id: String) -> Result<(), PasswordError> {
    let mut clipboard_guard = get_clipboard()
        .lock()
        .expect("another thread holding the lock paniced");

    match clipboard_guard.as_mut() {
        Some(clipboard) => match clipboard.set_text(pass_id) {
            Ok(()) => Ok(()),
            Err(e) => Err(PasswordError::ClipboardError(e)),
        },
        None => Err(PasswordError::ClipboardUnavailable),
    }
}

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

pub struct PasswordStore {
    pub passwords: Vec<PasswordInfo>,
    event_tx: Sender<PasswordEvent>,
    ops_map: HashMap<*const (), (JoinHandle<()>, String)>,
}

impl PasswordStore {
    pub fn new(event_tx: Sender<PasswordEvent>) -> Self {
        let store_dir = Self::get_store_dir();
        let mut passwords = Self::get_password_infos(&store_dir);
        passwords.sort_by_key(|element| element.pass_id.clone());
        Self {
            passwords,
            event_tx,
            ops_map: HashMap::new(),
        }
    }

    pub fn get_store_dir() -> PathBuf {
        let home = dirs::home_dir().expect("could not determine home directory");
        if let Some(store_path) = env::var_os("PASSWORD_STORE_DIR") {
            let path = PathBuf::from(store_path);
            if path.is_absolute() {
                return path;
            } else if let Ok(relative_to_home) = path
                .strip_prefix("$HOME")
                .or_else(|_| path.strip_prefix("~"))
            {
                return home.join(relative_to_home);
            };
        }
        home.join(".password-store")
    }

    pub fn get_password_infos(store_dir: &Path) -> Vec<PasswordInfo> {
        Self::read_store_dir(store_dir)
            .unwrap_or_default()
            .iter()
            .filter_map(|path| {
                let relative_path = path.strip_prefix(store_dir).expect("prefix does exist");
                match path.metadata() {
                    Ok(metadata) => Some(PasswordInfo::new(relative_path, metadata)),
                    Err(_) => None,
                }
            })
            .collect()
    }

    fn read_store_dir(dir: &Path) -> io::Result<Vec<PathBuf>> {
        let mut result = Vec::new();

        fn visit_dir(dir: &Path, result: &mut Vec<PathBuf>) -> io::Result<()> {
            for entry in fs::read_dir(dir)? {
                let path = entry?.path();
                if path.is_dir() {
                    visit_dir(&path, result)?;
                } else if path.is_file() && path.extension().is_some_and(|ext| ext == "gpg") {
                    result.push(path);
                }
            }
            Ok(())
        }

        visit_dir(dir, &mut result)?;
        Ok(result)
    }

    /// Runs the provided function in a new thread only if an operation
    /// with the same pass_id is not already running
    fn run_once(
        &mut self,
        pass_id: String,
        password_function: impl FnOnce(String) -> Result<PasswordEvent, PasswordError> + Send + 'static,
    ) {
        let fn_ptr = &password_function as *const _ as *const ();

        if let Some((handle, last_pass_id)) = self.ops_map.get(&fn_ptr) {
            if &pass_id == last_pass_id && !handle.is_finished() {
                return;
            }
        }

        let event_tx = self.event_tx.clone();
        let last_pass_id = pass_id.clone();

        let handle = thread::spawn(move || {
            let event = match password_function(pass_id) {
                Ok(event) => event,
                Err(error) => PasswordEvent::Status(Err(error)),
            };
            event_tx.send(event).expect("receiver deallocated");
        });

        self.ops_map.insert(fn_ptr, (handle, last_pass_id));
    }

    /// Runs in a new thread
    pub fn copy_password(&mut self, pass_id: String) {
        self.run_once(pass_id, copy_password);
    }

    /// Runs in a new thread
    pub fn copy_login(&mut self, pass_id: String) {
        self.run_once(pass_id, copy_login);
    }

    /// Runs in a new thread
    pub fn copy_otp(&mut self, pass_id: String) {
        self.run_once(pass_id, copy_otp);
    }

    /// Runs in a new thread
    pub fn fetch_otp(&mut self, pass_id: String) {
        self.run_once(pass_id, fetch_otp);
    }

    /// Runs in a new thread
    pub fn fetch_entry(&mut self, pass_id: String) {
        self.run_once(pass_id, fetch_entry);
    }
}
