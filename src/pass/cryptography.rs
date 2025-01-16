use gpgme::{Context, Protocol};
use std::cell::RefCell;

use crate::Error;

thread_local! {
    static GPG_CONTEXT: RefCell<Option<Context>> = const {RefCell::new(None)};
}

pub fn decrypt(cipher: &[u8]) -> Result<String, Error> {
    GPG_CONTEXT.with(|ctx| {
        let mut ctx_ref = ctx.borrow_mut();
        if ctx_ref.is_none() {
            *ctx_ref = Some(Context::from_protocol(Protocol::OpenPgp)?);
        }

        let ctx = ctx_ref.as_mut().unwrap();
        let mut plain = Vec::new();

        ctx.decrypt(cipher, &mut plain)?;

        String::from_utf8(plain).map_err(|e| e.into())
    })
}
