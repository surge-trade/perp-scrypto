use scrypto::prelude::*;

#[derive(ScryptoSbor, Clone, Default)]
pub struct Permissions {
    pub level_1: IndexSet<ComponentAddress>,
    pub level_2: IndexSet<ComponentAddress>,
    pub level_3: IndexSet<ComponentAddress>,
}
