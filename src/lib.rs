mod error;
mod event;
mod operations;
mod password_info;
mod password_store;

pub use error::PasswordError;
pub use event::PasswordEvent;
pub use operations::{copy_id, copy_login, copy_otp, copy_password, fetch_entry, fetch_otp};
pub use password_info::PasswordInfo;
pub use password_store::PasswordStore;

#[cfg(test)]
mod tests {
    #[test]
    fn asdf() {
        let result = 1;
        assert_eq!(result, 1);
    }
}
