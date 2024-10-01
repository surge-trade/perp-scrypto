use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub struct Input {
    pub vault: Vault,
    pub wrappable: bool,
}
