use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub struct AccountPosition {
    pub amount: Decimal,
    pub interest_checkpoint: Decimal,
}
