use std::convert::TryFrom;

use types::{
    bytesrepr::{self, FromBytes, ToBytes, U8_SERIALIZED_LENGTH},
    contracts::ContractPackage,
    CLValue, Contract, ContractWasm,
};

use crate::{account::Account, TypeMismatch};

#[repr(u8)]
enum Tag {
    CLValue = 0,
    Account = 1,
    ContractWasm = 2,
    Contract = 3,
    ContractPackage = 4,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum StoredValue {
    CLValue(CLValue),
    Account(Account),
    ContractWasm(ContractWasm),
    Contract(Contract),
    ContractPackage(ContractPackage),
}

impl StoredValue {
    pub fn as_cl_value(&self) -> Option<&CLValue> {
        match self {
            StoredValue::CLValue(cl_value) => Some(cl_value),
            _ => None,
        }
    }

    pub fn as_account(&self) -> Option<&Account> {
        match self {
            StoredValue::Account(account) => Some(account),
            _ => None,
        }
    }

    pub fn as_contract(&self) -> Option<&Contract> {
        match self {
            StoredValue::Contract(contract) => Some(contract),
            _ => None,
        }
    }

    pub fn as_contract_wasm(&self) -> Option<&ContractWasm> {
        match self {
            StoredValue::ContractWasm(contract_wasm) => Some(contract_wasm),
            _ => None,
        }
    }

    pub fn as_contract_package(&self) -> Option<&ContractPackage> {
        match self {
            StoredValue::ContractPackage(contract_package) => Some(&contract_package),
            _ => None,
        }
    }

    pub fn type_name(&self) -> String {
        match self {
            StoredValue::CLValue(cl_value) => format!("{:?}", cl_value.cl_type()),
            StoredValue::Account(_) => "Account".to_string(),
            StoredValue::ContractWasm(_) => "Contract".to_string(),
            StoredValue::Contract(_) => "Contract".to_string(),
            StoredValue::ContractPackage(_) => "ContractPackage".to_string(),
        }
    }
}

impl TryFrom<StoredValue> for CLValue {
    type Error = TypeMismatch;

    fn try_from(stored_value: StoredValue) -> Result<Self, Self::Error> {
        match stored_value {
            StoredValue::CLValue(cl_value) => Ok(cl_value),
            _ => Err(TypeMismatch::new(
                "CLValue".to_string(),
                stored_value.type_name(),
            )),
        }
    }
}

impl TryFrom<StoredValue> for Account {
    type Error = TypeMismatch;

    fn try_from(stored_value: StoredValue) -> Result<Self, Self::Error> {
        match stored_value {
            StoredValue::Account(account) => Ok(account),
            _ => Err(TypeMismatch::new(
                "Account".to_string(),
                stored_value.type_name(),
            )),
        }
    }
}

impl TryFrom<StoredValue> for ContractWasm {
    type Error = TypeMismatch;

    fn try_from(stored_value: StoredValue) -> Result<Self, Self::Error> {
        match stored_value {
            StoredValue::ContractWasm(contract_wasm) => Ok(contract_wasm),
            _ => Err(TypeMismatch::new(
                "ContractWasm".to_string(),
                stored_value.type_name(),
            )),
        }
    }
}

impl TryFrom<StoredValue> for ContractPackage {
    type Error = TypeMismatch;

    fn try_from(stored_value: StoredValue) -> Result<Self, Self::Error> {
        match stored_value {
            StoredValue::ContractPackage(contract_package) => Ok(contract_package),
            _ => Err(TypeMismatch::new(
                "ContractPackage".to_string(),
                stored_value.type_name(),
            )),
        }
    }
}

impl TryFrom<StoredValue> for Contract {
    type Error = TypeMismatch;

    fn try_from(stored_value: StoredValue) -> Result<Self, Self::Error> {
        match stored_value {
            StoredValue::Contract(contract) => Ok(contract),
            _ => Err(TypeMismatch::new(
                "Contract".to_string(),
                stored_value.type_name(),
            )),
        }
    }
}

impl ToBytes for StoredValue {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        let mut result = bytesrepr::allocate_buffer(self)?;
        let (tag, mut serialized_data) = match self {
            StoredValue::CLValue(cl_value) => (Tag::CLValue, cl_value.to_bytes()?),
            StoredValue::Account(account) => (Tag::Account, account.to_bytes()?),
            StoredValue::ContractWasm(contract_wasm) => {
                (Tag::ContractWasm, contract_wasm.to_bytes()?)
            }
            StoredValue::Contract(contract_header) => (Tag::Contract, contract_header.to_bytes()?),
            StoredValue::ContractPackage(contract_package) => {
                (Tag::ContractPackage, contract_package.to_bytes()?)
            }
        };
        result.push(tag as u8);
        result.append(&mut serialized_data);
        Ok(result)
    }

    fn serialized_length(&self) -> usize {
        U8_SERIALIZED_LENGTH
            + match self {
                StoredValue::CLValue(cl_value) => cl_value.serialized_length(),
                StoredValue::Account(account) => account.serialized_length(),
                StoredValue::ContractWasm(contract_wasm) => contract_wasm.serialized_length(),
                StoredValue::Contract(contract_header) => contract_header.serialized_length(),
                StoredValue::ContractPackage(contract_package) => {
                    contract_package.serialized_length()
                }
            }
    }
}

impl FromBytes for StoredValue {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (tag, remainder): (u8, &[u8]) = FromBytes::from_bytes(bytes)?;
        match tag {
            tag if tag == Tag::CLValue as u8 => CLValue::from_bytes(remainder)
                .map(|(cl_value, remainder)| (StoredValue::CLValue(cl_value), remainder)),
            tag if tag == Tag::Account as u8 => Account::from_bytes(remainder)
                .map(|(account, remainder)| (StoredValue::Account(account), remainder)),
            tag if tag == Tag::ContractWasm as u8 => {
                ContractWasm::from_bytes(remainder).map(|(contract_wasm, remainder)| {
                    (StoredValue::ContractWasm(contract_wasm), remainder)
                })
            }
            tag if tag == Tag::ContractPackage as u8 => {
                ContractPackage::from_bytes(remainder).map(|(contract_package, remainder)| {
                    (StoredValue::ContractPackage(contract_package), remainder)
                })
            }
            tag if tag == Tag::Contract as u8 => Contract::from_bytes(remainder)
                .map(|(contract, remainder)| (StoredValue::Contract(contract), remainder)),
            _ => Err(bytesrepr::Error::Formatting),
        }
    }
}

pub mod gens {
    use proptest::prelude::*;

    use types::gens::cl_value_arb;

    use super::StoredValue;
    use crate::account::gens::account_arb;
    use types::gens::{contract_arb, contract_package_arb, contract_wasm_arb};

    pub fn stored_value_arb() -> impl Strategy<Value = StoredValue> {
        prop_oneof![
            cl_value_arb().prop_map(StoredValue::CLValue),
            account_arb().prop_map(StoredValue::Account),
            contract_package_arb().prop_map(StoredValue::ContractPackage),
            contract_arb().prop_map(StoredValue::Contract),
            contract_wasm_arb().prop_map(StoredValue::ContractWasm),
        ]
    }
}

#[cfg(test)]
mod tests {
    use proptest::proptest;

    use super::*;

    proptest! {
        #[test]
        fn serialization_roundtrip(v in gens::stored_value_arb()) {
            bytesrepr::test_serialization_roundtrip(&v);
        }
    }
}
