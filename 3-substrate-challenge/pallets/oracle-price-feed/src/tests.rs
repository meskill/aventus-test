use core::str;
use std::sync::Arc;

use super::*;
use crate as pallet_oracle_price_feed;
use frame::deps::{
    frame_support,
    frame_system::{self},
    sp_core::offchain::{
        testing::{OffchainState, PendingRequest, TestOffchainExt},
        OffchainDbExt, OffchainWorkerExt,
    },
};
use frame::{deps::sp_io::TestExternalities, testing_prelude::*};
use parking_lot::RwLock;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        OraclePriceFeedGold: pallet_oracle_price_feed,
        OraclePriceFeedSilver: pallet_oracle_price_feed::<Instance1>,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
}

parameter_types! {
    // don't forget to start the server with `cargo run -p external-api`
    pub const OraclePriceFeedApiEndpointGold: &'static str = "http://localhost:8000/updatePrice";
    pub const OraclePriceFeedApiEndpointSilver: &'static str = "http://localhost:9000/updatePrice";
}

// Implement the pallet's Config trait
impl pallet_oracle_price_feed::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type PriceApiEndpoint = OraclePriceFeedApiEndpointGold;
}

impl pallet_oracle_price_feed::Config<Instance1> for Test {
    type RuntimeEvent = RuntimeEvent;
    type PriceApiEndpoint = OraclePriceFeedApiEndpointSilver;
}

fn new_ext() -> (TestExternalities, Arc<RwLock<OffchainState>>) {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    let (offchain, state) = TestOffchainExt::new();
    let mut t = TestExternalities::new(storage);
    t.register_extension(OffchainWorkerExt::new(offchain.clone()));
    t.register_extension(OffchainDbExt::new(offchain));

    (t, state)
}

const BLOCK_HASH: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
const STORAGE_KEY: &str = "0x5f2709552ceda2ce112547d1d04715c2a5077b5d721a7e0f155da5acd563a9ff";
const STORAGE_KEY_1: &str = "0x3dac2036f4656d3ad091463cc0d07f9ba5077b5d721a7e0f155da5acd563a9ff";

fn pending_request(instance: usize, block_number: usize, price: FixedU128) -> PendingRequest {
    let (uri, storage_key) = match instance {
        0 => (
            OraclePriceFeedApiEndpointGold::get().to_string(),
            STORAGE_KEY,
        ),
        1 => (
            OraclePriceFeedApiEndpointSilver::get().to_string(),
            STORAGE_KEY_1,
        ),
        _ => panic!("No mock for instance"),
    };

    PendingRequest {
		method: "POST".into(),
		uri,
		response: Some(b"Accepted".to_vec()),
		headers: vec![("content-type".to_string(), "application/json".to_string())],
		body: format!(r#"{{"blockHash":"{BLOCK_HASH}","blockNumber":"0x{block_number:x}","price":"{}","storageKey":"{storage_key}"}}"#, price.to_string()).as_bytes().to_vec(),
		sent: true,
		..Default::default()
	}
}

#[test]
fn sends_the_price() {
    let (mut t, state) = new_ext();

    t.execute_with(|| {
        System::set_block_number(1);
        let price = Price::from_float(123.12);
        pallet::Price::<Test>::put(price);

        state.write().expect_request(pending_request(0, 1, price));

        pallet::Pallet::<Test>::offchain_worker(1);

        System::assert_last_event(pallet::Event::<Test>::SendPrice(price).into());
    });
}

#[test]
fn resends_the_price_only_on_change() {
    let (mut t, state) = new_ext();

    t.execute_with(|| {
        let price = Price::from_float(123.12);
        pallet::Price::<Test>::put(price);

        state.write().expect_request(pending_request(0, 1, price));

        System::set_block_number(1);
        pallet::Pallet::<Test>::offchain_worker(1);

        System::set_block_number(2);
        pallet::Pallet::<Test>::offchain_worker(2);

        System::set_block_number(3);
        pallet::Pallet::<Test>::offchain_worker(3);

        let price = Price::from_float(123.38);
        pallet::Price::<Test>::put(price);

        state.write().expect_request(pending_request(0, 4, price));

        System::set_block_number(4);
        pallet::Pallet::<Test>::offchain_worker(4);

        System::assert_last_event(pallet::Event::<Test>::SendPrice(price).into());
        assert_eq!(System::event_count(), 2);
    });
}

#[test]
fn multiple_instances_are_independant() {
    let (mut t, state) = new_ext();

    t.execute_with(|| {
        let price = Price::from_float(15456.56);
        pallet::Price::<Test>::put(price);

        state.write().expect_request(pending_request(0, 1, price));

        System::set_block_number(1);
        pallet::Pallet::<Test>::offchain_worker(1);
        pallet::Pallet::<Test, Instance1>::offchain_worker(1);

        assert_eq!(System::event_count(), 1);

        let price = Price::from_float(15456.56);
        pallet::Price::<Test, Instance1>::put(price);

        state.write().expect_request(pending_request(1, 2, price));

        System::set_block_number(2);
        pallet::Pallet::<Test>::offchain_worker(2);
        pallet::Pallet::<Test, Instance1>::offchain_worker(2);

        assert_eq!(System::event_count(), 2);
        System::assert_last_event(pallet::Event::<Test, Instance1>::SendPrice(price).into());

        System::set_block_number(3);
        pallet::Pallet::<Test>::offchain_worker(3);
        pallet::Pallet::<Test, Instance1>::offchain_worker(3);

        assert_eq!(System::event_count(), 2);

        let price = Price::from_float(256846.18);
        pallet::Price::<Test>::put(price);

        state.write().expect_request(pending_request(0, 4, price));

        System::set_block_number(4);
        pallet::Pallet::<Test>::offchain_worker(4);
        pallet::Pallet::<Test, Instance1>::offchain_worker(4);

        assert_eq!(System::event_count(), 3);

        System::set_block_number(5);
        pallet::Pallet::<Test>::offchain_worker(5);
        pallet::Pallet::<Test, Instance1>::offchain_worker(5);

        assert_eq!(System::event_count(), 3);
    });
}
