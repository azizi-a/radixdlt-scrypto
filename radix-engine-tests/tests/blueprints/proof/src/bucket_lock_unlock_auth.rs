use scrypto::api::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, NonFungibleData)]
pub struct Example {
    pub name: String,
    #[mutable]
    pub available: bool,
}

#[blueprint]
mod bucket_lock_unlock_auth {
    struct BucketLockUnlockAuth {
        bucket: Bucket,
    }

    impl BucketLockUnlockAuth {
        pub fn call_lock_fungible_amount_directly() {
            let bucket = ResourceBuilder::new_fungible().mint_initial_supply(100);

            ScryptoEnv
                .call_method(
                    bucket.0.as_node_id(),
                    "lock_fungible_amount",
                    scrypto_args!(Decimal::from(1)),
                )
                .unwrap();
        }

        pub fn call_unlock_fungible_amount_directly() {
            let bucket = ResourceBuilder::new_fungible().mint_initial_supply(100);

            let _proof = bucket.create_proof();

            ScryptoEnv
                .call_method(
                    bucket.0.as_node_id(),
                    "unlock_fungible_amount",
                    scrypto_args!(Decimal::from(1)),
                )
                .unwrap();
        }

        pub fn call_lock_non_fungibles_directly() {
            let bucket = ResourceBuilder::new_integer_non_fungible().mint_initial_supply([(
                1u64.into(),
                Example {
                    name: "One".to_owned(),
                    available: true,
                },
            )]);

            ScryptoEnv
                .call_method(
                    bucket.0.as_node_id(),
                    "lock_non_fungibles",
                    scrypto_args!([NonFungibleLocalId::integer(1)]),
                )
                .unwrap();
        }

        pub fn call_unlock_non_fungibles_directly() {
            let bucket = ResourceBuilder::new_integer_non_fungible().mint_initial_supply([(
                1u64.into(),
                Example {
                    name: "One".to_owned(),
                    available: true,
                },
            )]);

            let _proof = bucket.create_proof();

            ScryptoEnv
                .call_method(
                    bucket.0.as_node_id(),
                    "unlock_non_fungibles",
                    scrypto_args!([NonFungibleLocalId::integer(1)]),
                )
                .unwrap();
        }
    }
}
