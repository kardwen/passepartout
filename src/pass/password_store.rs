use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use super::PasswordInfo;

/// A password store that manages password entries.
#[derive(Default)]
pub struct PasswordStore {
    pub store_dir: PathBuf,
    pub passwords: Vec<PasswordInfo>,
}

impl PasswordStore {
    /// Creates a new password store instance with loaded password entries.
    ///
    /// Initializes the store by reading all password entries from the password store
    /// directory, sorting them by ID.
    pub fn new() -> Self {
        let store_dir = Self::get_store_dir();
        let mut passwords = Self::get_password_infos(&store_dir);
        passwords.sort_by_key(|info| info.id.clone());
        Self {
            store_dir,
            passwords,
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
                // Get pass ID
                let pass_id = path
                    .strip_prefix(store_dir)
                    .expect("store_dir should be a prefix")
                    .with_extension("")
                    .to_string_lossy()
                    .into();

                match path.metadata() {
                    Ok(metadata) => Some(PasswordInfo::new(pass_id, metadata)),
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
                } else if path.is_file()
                    && path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("gpg"))
                {
                    result.push(path);
                }
            }
            Ok(())
        }

        visit_dir(dir, &mut result)?;
        Ok(result)
    }
}
