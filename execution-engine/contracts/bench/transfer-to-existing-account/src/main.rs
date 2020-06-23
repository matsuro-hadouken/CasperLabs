#![no_std]
#![no_main]

use contract::{
    contract_api::{runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{account::AccountHash, ApiError, TransferredTo, U512};

const ARG_TARGET: &str = "target";
const ARG_AMOUNT: &str = "amount";

#[repr(u16)]
enum Error {
    TransferredToNewAccount = 0,
}

#[no_mangle]
pub extern "C" fn call() {
    let account: AccountHash = runtime::get_named_arg(ARG_TARGET);
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    let result = system::transfer_to_account(account, amount).unwrap_or_revert();
    match result {
        TransferredTo::ExistingAccount => {
            // This is the expected result, as all accounts have to be initialized beforehand
        }
        TransferredTo::NewAccount => {
            runtime::revert(ApiError::User(Error::TransferredToNewAccount as u16))
        }
    }
}
