#[cfg(feature = "gpgme")]
mod gpgme;

#[cfg(feature = "sequoia")]
mod sequoia;

use crate::Error;

pub fn decrypt(cipher: &[u8]) -> Result<String, Error> {
    #[cfg(feature = "gpgme")]
    return gpgme::decrypt(cipher);

    #[cfg(feature = "sequoia")]
    return sequoia::decrypt(cipher);
}
