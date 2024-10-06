#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(test)]
mod tests;

use frame::deps::sp_core::offchain::Duration;
use frame::deps::sp_io::offchain::timestamp;
use frame::deps::sp_runtime::offchain::{http, storage::StorageValueRef};
use frame::{
    arithmetic::FixedU128,
    deps::sp_core::hexdisplay::{AsBytesRef, HexDisplay},
};
use frame::{log, prelude::*};

use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

// Re-export all pallet parts, this is needed to properly import the pallet into the runtime.
pub use pallet::*;

pub type Price = FixedU128;

struct Payload {
    price: Price,
    storage_key: String,
    block_number: String,
    block_hash: String,
}

fn hex<T: AsBytesRef>(value: &T) -> String {
    let hex = HexDisplay::from(value);
    format!("0x{}", hex)
}

#[frame::pallet]
pub mod pallet {

    use super::Price as PriceType;
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T, I = ()>(_);

    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        type RuntimeEvent: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        #[pallet::constant]
        type PriceApiEndpoint: Get<&'static str>;
    }

    #[pallet::storage]
    #[pallet::getter(fn price)]
    pub type Price<T: Config<I>, I: 'static = ()> = StorageValue<_, PriceType, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (crate) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        SetPrice(PriceType),
        SendPrice(PriceType),
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        JsonConversion,
        HttpError,
        DecodeError,
    }

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        #[pallet::call_index(0)]
        #[pallet::weight({0})]
        pub fn set_price(origin: OriginFor<T>, price: PriceType) -> DispatchResult {
            ensure_root(origin)?;
            Price::<T, I>::put(price);
            Self::deposit_event(Event::SetPrice(price));
            Ok(())
        }
    }

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            if let Err(error) = Self::process(block_number) {
                log::error!("Failed to process price due to error: {:?}", error)
            }
        }
    }

    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        fn process(block_number: BlockNumberFor<T>) -> DispatchResult {
            let price = Price::<T, I>::get();

            if let Some(price) = price {
                log::debug!("Price is {}", price);

                let local_storage_name = Self::get_local_storage_entry_name();
                let local_storage = StorageValueRef::persistent(&local_storage_name);

                let local_value = local_storage
                    .get::<PriceType>()
                    .map_err(|_| Error::<T, I>::DecodeError)?;

                if local_value == Some(price) {
                    log::info!("Not sending value since it is unchanged");
                    return Ok(());
                }

                log::info!("Send the value to the API");

                let block_hash = <frame_system::Pallet<T>>::block_hash(block_number);

                let payload = Payload {
                    price,
                    storage_key: hex(&Price::<T, I>::hashed_key()),
                    block_number: format!("0x{:x}", block_number.into()),
                    block_hash: hex(&block_hash.as_ref()),
                };

                match Self::send_price(payload) {
                    Ok(_) => {
                        // set to local storage and generate event only on successful send
                        local_storage.set(&price);
                        Self::deposit_event(Event::SendPrice(price));
                    }
                    Err(error) => log::error!("Failed to send request due to error: {:?}", error),
                };
            }

            Ok(())
        }

        fn send_price(payload: Payload) -> DispatchResult {
            let deadline = timestamp().add(Duration::from_millis(2_000));

            let url = T::PriceApiEndpoint::get();

            let post_data = serde_json::json!({
                "price": payload.price,
                "storageKey": payload.storage_key,
                "blockNumber": payload.block_number,
                "blockHash": payload.block_hash
            });
            let body = serde_json::to_vec(&post_data).map_err(|_| Error::<T, I>::JsonConversion)?;

            let request =
                http::Request::post(url, vec![body]).add_header("content-type", "application/json");

            let pending = request.deadline(deadline).send().map_err(|err| {
                log::error!("Http request error: {:?}", err);
                Error::<T, I>::HttpError
            })?;
            let response = pending
                .try_wait(deadline)
                .map_err(|err| {
                    log::error!("Http pending error: {:?}", err);
                    Error::<T, I>::HttpError
                })?
                .map_err(|err| {
                    log::error!("Http response error: {:?}", err);
                    Error::<T, I>::HttpError
                })?;

            log::info!("Got response status: {}", response.code);

            ensure!(response.code == 200, Error::<T, I>::HttpError);

            let body = response.body().collect::<Vec<u8>>();

            let body_str = alloc::str::from_utf8(&body).map_err(|_| {
                log::warn!("Response parsing failed");
                Error::<T, I>::JsonConversion
            })?;

            log::info!("Response: {}", body_str);

            Ok(())
        }

        fn get_local_storage_entry_name() -> Vec<u8> {
            let instance_name = core::any::type_name::<I>();
            let mut name = b"oracle-price-feed::price::".to_vec();
            // extend the name with the api endpoint to avoid name
            // clashing of the local storage value for different instances
            name.extend(instance_name.as_bytes());

            name
        }
    }
}
