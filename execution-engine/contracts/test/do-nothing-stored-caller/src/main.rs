#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

use contract::contract_api::runtime;
use types::{runtime_args, ContractPackageHash, RuntimeArgs};

const ENTRY_FUNCTION_NAME: &str = "delegate";
const PURSE_NAME_ARG_NAME: &str = "purse_name";
const ARG_CONTRACT_PACKAGE: &str = "contract_package";
const ARG_NEW_PURSE_NAME: &str = "new_purse_name";
const ARG_VERSION: &str = "version";

#[no_mangle]
pub extern "C" fn call() {
    let contract_package_hash: ContractPackageHash = runtime::get_named_arg(ARG_CONTRACT_PACKAGE);
    let new_purse_name: String = runtime::get_named_arg(ARG_NEW_PURSE_NAME);
    let version_number: u8 = runtime::get_named_arg(ARG_VERSION);

    let args = runtime_args! {
        PURSE_NAME_ARG_NAME => new_purse_name,
    };

    runtime::call_versioned_contract(
        contract_package_hash,
        version_number,
        ENTRY_FUNCTION_NAME,
        args,
    )
}