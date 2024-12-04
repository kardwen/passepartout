mod error;
mod events;
mod password_info;
mod password_store;

pub use error::PasswordError;
pub use events::PasswordEvent;
pub use password_info::PasswordInfo;
pub use password_store::PasswordStore;
pub use password_store::{copy_id, copy_login, copy_otp, copy_password, fetch_entry, fetch_otp};

#[cfg(test)]
mod tests {
    #[test]
    fn asdf() {
        let result = 1;
        assert_eq!(result, 1);
    }
}
