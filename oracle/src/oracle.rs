use scrypto::prelude::*;
use common::PairId;

#[blueprint]
mod oracle {
    struct Oracle {
        public_key: Bls12381G1PublicKey,
        counter: PairId,
        pair_ids: HashMap<String, PairId>,
        prices: HashMap<PairId, Decimal>,
    }

    impl Oracle {
        pub fn new(owner_role: OwnerRole, public_key: Bls12381G1PublicKey) -> Global<Oracle> {    
            Self {
                public_key,
                counter: 0,
                pair_ids: HashMap::new(),
                prices: HashMap::new()
            }
            .instantiate()  
            .prepare_to_globalize(owner_role)
            .globalize()
        }

        pub fn get_pair_id(&mut self, pair_name: String) -> PairId {
            if let Some(pair_id) = self.pair_ids.get(&pair_name) {
                *pair_id
            } else {
                let pair_id = self.counter;
                self.pair_ids.insert(pair_name, pair_id);
                self.counter += 1;
                pair_id
            }
        }

        pub fn push_price(&mut self, pair_id: PairId, price: Decimal) {
            self.prices.insert(pair_id, price);
        }

        pub fn push_prices(&mut self, data: Vec<u8>, signature: Bls12381G2Signature) {
            let hash = CryptoUtils::keccak256_hash(data).to_vec();
            assert!(
                CryptoUtils::bls12381_v1_verify(hash, self.public_key, signature),
                "Invalid signature."
            );

            // for (pair_name, price) in prices.into_iter() {
            //     let pair_id = self.get_pair_id(pair_name);
            //     self.prices.insert(pair_id, price);
            // }
        }

        pub fn prices(&self, _max_age: Instant) -> HashMap<PairId, Decimal> {
            self.prices.clone()
        }
    }
}