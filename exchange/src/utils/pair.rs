use scrypto::prelude::*;

#[derive(ScryptoSbor, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pair(u32, u32);