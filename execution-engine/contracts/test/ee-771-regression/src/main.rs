#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;

use contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use types::{
    contracts::{NamedKeys, Parameters},
    ApiError, CLType, ContractHash, ContractVersion, EntryPoint, EntryPointAccess, EntryPointType,
    EntryPoints, RuntimeArgs,
};

const ENTRY_POINT_NAME: &str = "contract_ext";
const CONTRACT_KEY: &str = "contract";

#[no_mangle]
pub extern "C" fn contract_ext() {
    match runtime::get_key(CONTRACT_KEY) {
        Some(contract_key) => {
            // Calls a stored contract if exists.
            runtime::call_contract(
                contract_key.into_hash().expect("should be a hash"),
                "contract_ext",
                RuntimeArgs::default(),
            )
        }
        None => {
            // If given key doesn't exist it's the tail call, and an error is triggered.
            let entry_points = {
                let mut entry_points = EntryPoints::new();

                let entry_point = EntryPoint::new(
                    "functiondoesnotexist",
                    Parameters::default(),
                    CLType::Unit,
                    EntryPointAccess::Public,
                    EntryPointType::Contract,
                );

                entry_points.add_entry_point(entry_point);

                entry_points
            };
            storage::new_contract(entry_points, None, None, None);
        }
    }
}

fn store(named_keys: NamedKeys) -> (ContractHash, ContractVersion) {
    // extern "C" fn call(named_keys: NamedKeys) {
    let entry_points = {
        let mut entry_points = EntryPoints::new();

        let entry_point = EntryPoint::new(
            ENTRY_POINT_NAME,
            Parameters::default(),
            CLType::Unit,
            EntryPointAccess::Public,
            EntryPointType::Contract,
        );

        entry_points.add_entry_point(entry_point);

        entry_points
    };
    storage::new_contract(entry_points, Some(named_keys), None, None)
}

fn install() -> Result<ContractHash, ApiError> {
    let (contract_hash, _contract_version) = store(NamedKeys::new());

    let mut keys = NamedKeys::new();
    keys.insert(CONTRACT_KEY.to_string(), contract_hash.into());
    let (contract_hash, _contract_version) = store(keys);

    let mut keys_2 = NamedKeys::new();
    keys_2.insert(CONTRACT_KEY.to_string(), contract_hash.into());
    let (contract_hash, _contract_version) = store(keys_2);

    runtime::put_key(CONTRACT_KEY, contract_hash.into());

    Ok(contract_hash)
}

fn dispatch(contract_hash: ContractHash) {
    runtime::call_contract(contract_hash, "contract_ext", RuntimeArgs::default())
}

#[no_mangle]
pub extern "C" fn call() {
    let contract_key = install().unwrap_or_revert();
    dispatch(contract_key)
}
