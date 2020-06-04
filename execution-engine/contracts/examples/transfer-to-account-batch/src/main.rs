#![no_std]
#![no_main]

extern crate alloc;

use alloc::{collections::BTreeMap, vec::Vec};
use contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{account::AccountHash, ApiError, Key, U512};

const FN_NAME: &str = "transfer_batch";

/// Assumes 0-th argument is a Vec<(AccountHash, U512)>.
/// Performs a transfer for each element of the vector,
/// sending the specified amount to the specified key.
#[no_mangle]
pub extern "C" fn transfer_batch() {
    let transfers: Vec<(AccountHash, U512)> = runtime::get_arg(0)
        .unwrap_or_revert_with(ApiError::MissingArgument)
        .unwrap_or_revert_with(ApiError::InvalidArgument);

    for (account_hash, amount) in transfers {
        system::transfer_to_account(account_hash, amount).unwrap_or_revert();
    }
}

fn store_at_hash() -> Key {
    let pointer = storage::store_function_at_hash(FN_NAME, BTreeMap::new());
    pointer.into()
}

#[no_mangle]
pub extern "C" fn call() {
    let key = store_at_hash();
    runtime::put_key(FN_NAME, key);
}