use scrypto::prelude::*;

#[derive(ScryptoSbor, Default)]
pub struct AccountPosition {
    pub amount: Decimal,
    pub interest_checkpoint: Decimal,
}
