use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub struct AccountPosition {
    pub amount: Decimal,
    pub funding_index: Decimal,
}
