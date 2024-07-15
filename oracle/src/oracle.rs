mod errors;

use scrypto::prelude::*;
use common::{PairId, ListIndex, _AUTHORITY_RESOURCE};
use self::errors::*;

#[derive(ScryptoSbor)]
pub struct Price {
    pub pair: String,
    pub quote: Decimal,
    pub timestamp: Instant,
}

#[blueprint]
mod oracle_mod {
    const AUTHORITY_RESOURCE: ResourceAddress = _AUTHORITY_RESOURCE;

    enable_method_auth!(
        roles {
            authority => updatable_by: [];
            push_user => updatable_by: [];
            get_user => updatable_by: [];
        },
        methods {
            add_key => restrict_to: [OWNER];
            remove_key => restrict_to: [OWNER];
            remove_price => restrict_to: [OWNER];
            push_and_get_prices_with_auth => restrict_to: [authority];
            get_prices_with_auth => restrict_to: [authority];
            push_and_get_prices => restrict_to: [push_user];
            get_prices => restrict_to: [get_user];
        }
    );

    struct Oracle {
        keys: HashMap<ListIndex, Bls12381G1PublicKey>,
        prices: HashMap<PairId, (Decimal, Instant)>,
    }

    impl Oracle {
        pub fn new(owner_role: OwnerRole, keys: HashMap<ListIndex, Bls12381G1PublicKey>) -> Global<Oracle> {    
            Self {
                keys,
                prices: HashMap::new()
            }
            .instantiate()  
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                authority => rule!(require(AUTHORITY_RESOURCE));
                push_user => rule!(allow_all);
                get_user => rule!(allow_all);
            })
            .enable_component_royalties(component_royalties! {
                roles {
                    royalty_setter => OWNER;
                    royalty_setter_updater => OWNER;
                    royalty_locker => OWNER;
                    royalty_locker_updater => rule!(deny_all);
                    royalty_claimer => OWNER;
                    royalty_claimer_updater => OWNER;
                },
                init {
                    add_key => Free, locked;
                    remove_key => Free, locked;
                    remove_price => Free, locked;
                    push_and_get_prices_with_auth => Free, locked;
                    get_prices_with_auth => Free, locked;
                    push_and_get_prices => Usd(dec!(0.1)), updatable;
                    get_prices => Usd(dec!(0.1)), updatable;
                }
            })
            .globalize()
        }

        pub fn add_key(&mut self, id: ListIndex, public_key: Bls12381G1PublicKey) {
            assert!(
                !self.keys.contains_key(&id),
                "{}", ERROR_KEY_ID_ALREADY_EXISTS
            );

            self.keys.insert(id, public_key);
        }

        pub fn remove_key(&mut self, id: ListIndex) {
            self.keys.remove(&id);
        }

        pub fn remove_price(&mut self, pair_id: PairId) {
            self.prices.remove(&pair_id);
        }

        pub fn push_and_get_prices_with_auth(&mut self, pair_ids: HashSet<PairId>, max_age: Instant, data: Vec<u8>, signature: Bls12381G2Signature, key_id: ListIndex) -> HashMap<PairId, Decimal> {
            self._push_and_get_prices(pair_ids, max_age, data, signature, key_id)
        }

        pub fn get_prices_with_auth(&self, pair_ids: HashSet<PairId>, max_age: Instant) -> HashMap<PairId, Decimal> {
            self._get_prices(pair_ids, max_age)
        }

        pub fn push_and_get_prices(&mut self, pair_ids: HashSet<PairId>, max_age: Instant, data: Vec<u8>, signature: Bls12381G2Signature, key_id: ListIndex) -> HashMap<PairId, Decimal> {
            self._push_and_get_prices(pair_ids, max_age, data, signature, key_id)
        }

        pub fn get_prices(&self, pair_ids: HashSet<PairId>, max_age: Instant) -> HashMap<PairId, Decimal> {
            self._get_prices(pair_ids, max_age)
        }

        fn _push_and_get_prices(&mut self, pair_ids: HashSet<PairId>, max_age: Instant, data: Vec<u8>, signature: Bls12381G2Signature, key_id: ListIndex) -> HashMap<PairId, Decimal> {
            let prices: Vec<Price> = scrypto_decode(&data).expect(ERROR_INVALID_DATA);

            let hash = CryptoUtils::keccak256_hash(data).to_vec();
            let key = *self.keys.get(&key_id).expect(ERROR_INVALID_KEY_ID);
            assert!(
                CryptoUtils::bls12381_v1_verify(hash.clone(), key, signature),
                "{}", ERROR_INVALID_SIGNATURE
            );

            prices.into_iter().for_each(|p1| {
                self.prices.entry(p1.pair)
                    .and_modify(|p| {
                        if p1.timestamp.compare(p.1, TimeComparisonOperator::Gt) {
                            *p = (p1.quote, p1.timestamp);
                        }
                    })
                    .or_insert((p1.quote, p1.timestamp));
            });

            self.get_prices(pair_ids, max_age)
        }

        fn _get_prices(&self, pair_ids: HashSet<PairId>, max_age: Instant) -> HashMap<PairId, Decimal> {
            pair_ids.into_iter().map(|pair_id| {
                let (quote, timestamp) = *self.prices.get(&pair_id).expect(ERROR_MISSING_PAIR);
                assert!(
                    timestamp.compare(max_age, TimeComparisonOperator::Gt),
                    "{}", ERROR_PRICE_TOO_OLD
                );
                (pair_id, quote)
            }).collect()
        }
    }
}
