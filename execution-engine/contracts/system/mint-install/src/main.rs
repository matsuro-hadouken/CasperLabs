#![no_std]
#![no_main]

use contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{contracts::NamedKeys, CLValue};

const HASH_KEY_NAME: &str = "mint_hash";
const ACCESS_KEY_NAME: &str = "mint_access";

#[no_mangle]
pub extern "C" fn mint() {
    mint_token::mint();
}

#[no_mangle]
pub extern "C" fn create() {
    mint_token::create();
}

#[no_mangle]
pub extern "C" fn balance() {
    mint_token::balance();
}

#[no_mangle]
pub extern "C" fn transfer() {
    mint_token::transfer();
}

#[no_mangle]
pub extern "C" fn install() {
    let entry_points = mint_token::get_entry_points();

    let (contract_package_hash, access_uref) = storage::create_contract_package_at_hash();
    runtime::put_key(HASH_KEY_NAME, contract_package_hash.into());
    runtime::put_key(ACCESS_KEY_NAME, access_uref.into());

    let named_keys = NamedKeys::new();

    let (contract_key, _contract_version) =
        storage::add_contract_version(contract_package_hash, entry_points, named_keys);

    let return_value = CLValue::from_t((contract_package_hash, contract_key)).unwrap_or_revert();
    runtime::ret(return_value);
}
