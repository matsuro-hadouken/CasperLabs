#![no_std]

#[macro_use]
extern crate alloc;

use alloc::boxed::Box;

use contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use mint::{Mint, RuntimeProvider, StorageProvider};
use types::{
    account::AccountHash,
    bytesrepr::{FromBytes, ToBytes},
    contracts::Parameters,
    system_contract_errors::mint::Error,
    CLType, CLTyped, CLValue, EntryPoint, EntryPointAccess, EntryPointType, EntryPoints, Key,
    Parameter, URef, U512,
};

pub const METHOD_MINT: &str = "mint";
pub const METHOD_CREATE: &str = "create";
pub const METHOD_BALANCE: &str = "balance";
pub const METHOD_TRANSFER: &str = "transfer";

pub const ARG_AMOUNT: &str = "amount";
pub const ARG_PURSE: &str = "purse";
pub const ARG_SOURCE: &str = "source";
pub const ARG_TARGET: &str = "target";

pub struct MintContract;

impl RuntimeProvider for MintContract {
    fn get_caller(&self) -> AccountHash {
        runtime::get_caller()
    }

    fn put_key(&mut self, name: &str, key: Key) {
        runtime::put_key(name, key)
    }
}

impl StorageProvider for MintContract {
    fn new_uref<T: CLTyped + ToBytes>(&mut self, init: T) -> URef {
        storage::new_uref(init)
    }

    fn write_local<K: ToBytes, V: CLTyped + ToBytes>(&mut self, key: K, value: V) {
        storage::write_local(key, value)
    }

    fn read_local<K: ToBytes, V: CLTyped + FromBytes>(
        &mut self,
        key: &K,
    ) -> Result<Option<V>, Error> {
        storage::read_local(key).map_err(|_| Error::Storage)
    }

    fn read<T: CLTyped + FromBytes>(&mut self, uref: URef) -> Result<Option<T>, Error> {
        storage::read(uref).map_err(|_| Error::Storage)
    }

    fn write<T: CLTyped + ToBytes>(&mut self, uref: URef, value: T) -> Result<(), Error> {
        storage::write(uref, value);
        Ok(())
    }

    fn add<T: CLTyped + ToBytes>(&mut self, uref: URef, value: T) -> Result<(), Error> {
        storage::add(uref, value);
        Ok(())
    }
}

impl Mint for MintContract {}

pub fn mint() {
    let mut mint_contract = MintContract;
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    let result: Result<URef, Error> = mint_contract.mint(amount);
    let ret = CLValue::from_t(result).unwrap_or_revert();
    runtime::ret(ret)
}

pub fn create() {
    let mut mint_contract = MintContract;
    let uref = mint_contract.mint(U512::zero()).unwrap_or_revert();
    let ret = CLValue::from_t(uref).unwrap_or_revert();
    runtime::ret(ret)
}

pub fn balance() {
    let mut mint_contract = MintContract;
    let uref: URef = runtime::get_named_arg(ARG_PURSE);
    let balance: Option<U512> = mint_contract.balance(uref).unwrap_or_revert();
    let ret = CLValue::from_t(balance).unwrap_or_revert();
    runtime::ret(ret)
}

pub fn transfer() {
    let mut mint_contract = MintContract;
    let source: URef = runtime::get_named_arg(ARG_SOURCE);
    let target: URef = runtime::get_named_arg(ARG_TARGET);
    let amount: U512 = runtime::get_named_arg(ARG_AMOUNT);
    let result: Result<(), Error> = mint_contract.transfer(source, target, amount);
    let ret = CLValue::from_t(result).unwrap_or_revert();
    runtime::ret(ret);
}

pub fn get_entry_points() -> EntryPoints {
    let mut entry_points = EntryPoints::new();

    let entry_point = EntryPoint::new(
        METHOD_MINT,
        vec![Parameter::new(ARG_AMOUNT, CLType::U512)],
        CLType::Result {
            ok: Box::new(CLType::URef),
            err: Box::new(CLType::U8),
        },
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );
    entry_points.add_entry_point(entry_point);

    let entry_point = EntryPoint::new(
        METHOD_CREATE,
        Parameters::new(),
        CLType::URef,
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );
    entry_points.add_entry_point(entry_point);

    let entry_point = EntryPoint::new(
        METHOD_BALANCE,
        vec![Parameter::new(ARG_PURSE, CLType::URef)],
        CLType::Option(Box::new(CLType::U512)),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );
    entry_points.add_entry_point(entry_point);

    let entry_point = EntryPoint::new(
        METHOD_TRANSFER,
        vec![
            Parameter::new(ARG_SOURCE, CLType::URef),
            Parameter::new(ARG_TARGET, CLType::URef),
            Parameter::new(ARG_AMOUNT, CLType::U512),
        ],
        CLType::Result {
            ok: Box::new(CLType::Unit),
            err: Box::new(CLType::U8),
        },
        EntryPointAccess::Public,
        EntryPointType::Contract,
    );
    entry_points.add_entry_point(entry_point);

    entry_points
}
