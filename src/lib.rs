//! 🔑 Library for pass

mod clipboard;
mod error;
mod pass;

pub use error::Error;
pub use pass::{
    copy_id, copy_login, copy_otp, copy_password, decrypt_password_file, generate_otp,
    PasswordInfo, PasswordStore,
};

#[cfg(test)]
mod tests {
    #[test]
    fn asdf() {
        let result = 1;
        assert_eq!(result, 1);
    }
}
