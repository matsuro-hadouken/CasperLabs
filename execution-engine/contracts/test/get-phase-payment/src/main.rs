#![no_std]
#![no_main]

use contract::{
    contract_api::{account, runtime, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{Phase, RuntimeArgs, URef, U512};

const GET_PAYMENT_PURSE: &str = "get_payment_purse";
const ARG_PHASE: &str = "phase";

fn standard_payment(amount: U512) {
    let main_purse = account::get_main_purse();

    let pos_pointer = system::get_proof_of_stake();

    let payment_purse: URef =
        runtime::call_contract(pos_pointer, GET_PAYMENT_PURSE, RuntimeArgs::default());

    system::transfer_from_purse_to_purse(main_purse, payment_purse, amount).unwrap_or_revert()
}

#[no_mangle]
pub extern "C" fn call() {
    let known_phase: Phase = runtime::get_named_arg(ARG_PHASE);
    let get_phase = runtime::get_phase();
    assert_eq!(
        get_phase, known_phase,
        "get_phase did not return known_phase"
    );

    standard_payment(U512::from(10_000_000));
}
