#![no_std]
#![feature(cell_update)]

extern crate alloc;
extern crate contract_ffi;

use contract_ffi::contract_api::{
    get_arg, read_local, revert, transfer_to_account, write_local, Error,
};
use contract_ffi::value::account::PublicKey;
use contract_ffi::value::U512;

const TRANSFER_AMOUNT: u32 = 10_000_000;

/// Executes token transfer to supplied public key.
/// Transfers 10_000_000 motes every time.
///
/// Revert status codes:
/// 1 - requested transfer to already funded public key.
/// 2 - transfer error.
#[no_mangle]
pub extern "C" fn call() {
    let public_key: PublicKey = match get_arg(0) {
        Some(Ok(data)) => data,
        Some(Err(_)) => revert(Error::InvalidArgument),
        None => revert(Error::MissingArgument),
    };
    // Maybe we will decide to allow multiple funds up until some maximum value.
    let already_funded = read_local::<PublicKey, U512>(public_key)
        .unwrap_or_default()
        .is_some();
    if already_funded {
        revert(Error::User(1));
    } else {
        let u512_tokens = U512::from(TRANSFER_AMOUNT);
        if transfer_to_account(public_key, u512_tokens).is_err() {
            revert(Error::User(2))
        } else {
            // Transfer successful; Store the fact of funding in the local state.
            write_local(public_key, u512_tokens);
        }
    }
}
