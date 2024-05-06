mod errors;

use scrypto::prelude::*;
use common::PairId;
use self::errors::*;

#[derive(ScryptoSbor)]
pub struct Price {
    pair: String,
    quote: Decimal,
    timestamp: Instant,
}

#[blueprint]
mod oracle_mod {
    struct Oracle {
        public_key: Bls12381G1PublicKey,
        prices: HashMap<PairId, (Decimal, Instant)>,
    }

    impl Oracle {
        pub fn new(owner_role: OwnerRole, public_key: Bls12381G1PublicKey) -> Global<Oracle> {    
            Self {
                public_key,
                prices: HashMap::new()
            }
            .instantiate()  
            .prepare_to_globalize(owner_role)
            .globalize()
        }

        pub fn hash(&self, data: Vec<u8>) -> Vec<u8> {
            CryptoUtils::keccak256_hash(data).to_vec()
        }

        pub fn push_prices(&mut self, pair_ids: Vec<PairId>, max_age: Instant, data: Vec<u8>, signature: Bls12381G2Signature) -> HashMap<PairId, Decimal> {
            let prices: Vec<Price> = scrypto_decode(&data).expect(ERROR_INVALID_DATA);

            let hash = CryptoUtils::keccak256_hash(data).to_vec();
            assert!(
                CryptoUtils::bls12381_v1_verify(hash.clone(), self.public_key, signature),
                "{}", ERROR_INVALID_SIGNATURE
            );

            self.prices.extend(prices.into_iter().map(|p| (p.pair.to_owned(), (p.quote, p.timestamp))));

            self.prices(pair_ids, max_age)
        }

        pub fn prices(&self, pair_ids: Vec<PairId>, max_age: Instant) -> HashMap<PairId, Decimal> {
            pair_ids.iter().map(|pair_id| {
                let (quote, timestamp) = self.prices.get(pair_id).expect(ERROR_MISSING_PAIR);
                assert!(
                    timestamp.compare(max_age, TimeComparisonOperator::Gt),
                    "{}", ERROR_PRICE_TOO_OLD
                );
                (pair_id.to_owned(), *quote)
            }).collect()
        }
    }
}
