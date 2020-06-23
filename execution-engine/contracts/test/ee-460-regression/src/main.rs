#![no_std]
#![no_main]

use contract::contract_api::{runtime, system};
use types::{account::AccountHash, ApiError, U512};

const ARG_AMOUNT: &str = "amount";

#[no_mangle]
pub extern "C" fn call() {
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    let account_hash = AccountHash::new([42; 32]);
    let result = system::transfer_to_account(account_hash, amount);
    assert_eq!(result, Err(ApiError::Transfer))
}
