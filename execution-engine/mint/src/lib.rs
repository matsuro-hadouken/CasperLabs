#![cfg_attr(not(feature = "std"), no_std)]

mod runtime_provider;
mod storage_provider;

use core::convert::TryFrom;

use types::{account::AccountHash, system_contract_errors::mint::Error, Key, URef, U512};

pub use crate::{runtime_provider::RuntimeProvider, storage_provider::StorageProvider};

const SYSTEM_ACCOUNT: AccountHash = AccountHash::new([0; 32]);

pub trait Mint: RuntimeProvider + StorageProvider {
    fn mint(&mut self, initial_balance: U512) -> Result<URef, Error> {
        let caller = self.get_caller();
        if !initial_balance.is_zero() && caller != SYSTEM_ACCOUNT {
            return Err(Error::InvalidNonEmptyPurseCreation);
        }

        let balance_key: Key = self.new_uref(initial_balance).into();
        let purse_uref: URef = self.new_uref(());
        let purse_uref_name = purse_uref.remove_access_rights().as_string();

        // store balance uref so that the runtime knows the mint has full access
        self.put_key(&purse_uref_name, balance_key);

        // store association between purse id and balance uref
        self.write_local(purse_uref.addr(), balance_key);
        // self.write(purse_uref.addr(), Key::Hash)

        Ok(purse_uref)
    }

    fn balance(&mut self, purse: URef) -> Result<Option<U512>, Error> {
        let balance_uref: URef = match self.read_local(&purse.addr())? {
            Some(key) => TryFrom::<Key>::try_from(key).map_err(|_| Error::InvalidAccessRights)?,
            None => return Ok(None),
        };
        match self.read(balance_uref)? {
            some @ Some(_) => Ok(some),
            None => Err(Error::PurseNotFound),
        }
    }

    fn transfer(&mut self, source: URef, target: URef, amount: U512) -> Result<(), Error> {
        if !source.is_writeable() || !target.is_addable() {
            return Err(Error::InvalidAccessRights);
        }
        let source_balance: URef = match self.read_local(&source.addr())? {
            Some(key) => TryFrom::<Key>::try_from(key).map_err(|_| Error::InvalidAccessRights)?,
            None => return Err(Error::SourceNotFound),
        };
        let source_value: U512 = match self.read(source_balance)? {
            Some(source_value) => source_value,
            None => return Err(Error::SourceNotFound),
        };
        if amount > source_value {
            return Err(Error::InsufficientFunds);
        }
        let target_balance: URef = match self.read_local(&target.addr())? {
            Some(key) => TryFrom::<Key>::try_from(key).map_err(|_| Error::InvalidAccessRights)?,
            None => return Err(Error::DestNotFound),
        };
        self.write(source_balance, source_value - amount)?;
        self.add(target_balance, amount)?;
        Ok(())
    }
}
