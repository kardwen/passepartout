use crate::PasswordError;

#[derive(Debug)]
pub enum PasswordEvent {
    Status(Result<Option<String>, PasswordError>),
    PasswordInfo {
        pass_id: String,
        file_contents: String,
    },
    OneTimePassword {
        pass_id: String,
        one_time_password: String,
    },
}
