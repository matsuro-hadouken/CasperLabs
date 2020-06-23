#![no_std]
#![no_main]

use contract::contract_api::runtime;
use types::ApiError;
const ARG_NUMBER: &str = "number";

#[no_mangle]
pub extern "C" fn call() {
    let number: u32 = runtime::get_named_arg(ARG_NUMBER);
    runtime::revert(ApiError::User(number as u16));
}
