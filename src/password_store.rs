use std::{
    collections::HashMap,
    env, fs, io,
    path::{Path, PathBuf},
    sync::mpsc::Sender,
    thread::{self, JoinHandle},
};

use crate::{
    error::PasswordError,
    events::PasswordEvent,
    operations::{copy_login, copy_otp, copy_password, fetch_entry, fetch_otp},
    password_info::PasswordInfo,
};

/// A password store that manages password entries and asynchronous operations.
pub struct PasswordStore {
    pub passwords: Vec<PasswordInfo>,
    event_tx: Sender<PasswordEvent>,
    ops_map: HashMap<*const (), (JoinHandle<()>, String)>,
}

impl PasswordStore {
    /// Creates a new password store instance with loaded password entries.
    ///
    /// Initializes the store by reading all password entries from the password store directory,
    /// sorting them by ID, and setting up the event channel for asynchronous operations.
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

    /// Determines the password store directory path.
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

    /// Collects and processes all password entries from the store directory.
    ///
    /// Recursively traverses the store directory to find all `.gpg` files and creates
    /// [`PasswordInfo`] instances containing metadata for each entry.
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

    /// Executes a password operation in a new thread if not already running.
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

    /// Copies the password to the clipboard in a separate thread.
    ///
    /// The operation will only be executed if no other copy operation
    /// is currently running for the same password ID.
    pub fn copy_password(&mut self, pass_id: String) {
        self.run_once(pass_id, copy_password);
    }

    /// Copies the login information to the clipboard in a separate thread.
    ///
    /// The operation will only be executed if no other copy operation
    /// is currently running for the same password ID.
    pub fn copy_login(&mut self, pass_id: String) {
        self.run_once(pass_id, copy_login);
    }

    /// Copies the one-time password (OTP) to the clipboard in a separate thread.
    ///
    /// The operation will only be executed if no other copy operation
    /// is currently running for the same password ID.
    pub fn copy_otp(&mut self, pass_id: String) {
        self.run_once(pass_id, copy_otp);
    }

    /// Retrieves the one-time password (OTP) in a separate thread.
    ///
    /// The operation will only be executed if no other fetch operation
    /// is currently running for the same password ID.
    pub fn fetch_otp(&mut self, pass_id: String) {
        self.run_once(pass_id, fetch_otp);
    }

    /// Retrieves the password file contents in a separate thread.
    ///
    /// The operation will only be executed if no other fetch operation
    /// is currently running for the same password ID.
    pub fn fetch_entry(&mut self, pass_id: String) {
        self.run_once(pass_id, fetch_entry);
    }
}
