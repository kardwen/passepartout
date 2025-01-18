use futures::executor::block_on;
use sequoia_gpg_agent::Agent;
use sequoia_openpgp;
use std::cell::RefCell;

use crate::Error;

pub fn decrypt(cipher: &[u8]) -> Result<String, Error> {
    let future = async move { Agent::connect_to_default().await };
    let agent = block_on(future)?;

    todo!()
}
