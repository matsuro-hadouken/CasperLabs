use std::{collections::BTreeSet, path::Path};

use engine_core::{
    engine_state::{deploy_item::DeployItem, executable_deploy_item::ExecutableDeployItem},
    DeployHash,
};
use types::{
    account::AccountHash, bytesrepr::ToBytes, contracts::ContractVersion, ContractHash, HashAddr,
    RuntimeArgs,
};

use crate::internal::utils;

#[derive(Default)]
struct DeployItemData {
    pub address: Option<AccountHash>,
    pub payment_code: Option<ExecutableDeployItem>,
    pub session_code: Option<ExecutableDeployItem>,
    pub gas_price: u64,
    pub authorization_keys: BTreeSet<AccountHash>,
    pub deploy_hash: DeployHash,
}

pub struct DeployItemBuilder {
    deploy_item: DeployItemData,
}

impl DeployItemBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_address(mut self, address: AccountHash) -> Self {
        self.deploy_item.address = Some(address);
        self
    }

    pub fn with_payment_bytes(mut self, module_bytes: Vec<u8>, args: RuntimeArgs) -> Self {
        let args = Self::serialize_args(args);
        self.deploy_item.payment_code =
            Some(ExecutableDeployItem::ModuleBytes { module_bytes, args });
        self
    }

    pub fn with_empty_payment_bytes(self, args: RuntimeArgs) -> Self {
        self.with_payment_bytes(vec![], args)
    }

    pub fn with_payment_code<T: AsRef<Path>>(self, file_name: T, args: RuntimeArgs) -> Self {
        let module_bytes = utils::read_wasm_file_bytes(file_name);
        self.with_payment_bytes(module_bytes, args)
    }

    pub fn with_stored_payment_hash(
        mut self,
        hash: ContractHash,
        entry_point: &str,
        args: RuntimeArgs,
    ) -> Self {
        let args = Self::serialize_args(args);
        self.deploy_item.payment_code = Some(ExecutableDeployItem::StoredContractByHash {
            hash,
            entry_point: entry_point.into(),
            args,
        });
        self
    }

    pub fn with_stored_payment_named_key(
        mut self,
        uref_name: &str,
        entry_point_name: &str,
        args: RuntimeArgs,
    ) -> Self {
        let args = Self::serialize_args(args);
        self.deploy_item.payment_code = Some(ExecutableDeployItem::StoredContractByName {
            name: uref_name.to_owned(),
            entry_point: entry_point_name.into(),
            args,
        });
        self
    }

    pub fn with_session_bytes(mut self, module_bytes: Vec<u8>, args: RuntimeArgs) -> Self {
        let args = Self::serialize_args(args);
        self.deploy_item.session_code =
            Some(ExecutableDeployItem::ModuleBytes { module_bytes, args });
        self
    }

    pub fn with_session_code<T: AsRef<Path>>(self, file_name: T, args: RuntimeArgs) -> Self {
        let module_bytes = utils::read_wasm_file_bytes(file_name);
        self.with_session_bytes(module_bytes, args)
    }

    pub fn with_transfer_args(mut self, args: RuntimeArgs) -> Self {
        let args = Self::serialize_args(args);
        self.deploy_item.session_code = Some(ExecutableDeployItem::Transfer { args });
        self
    }

    pub fn with_stored_session_hash(
        mut self,
        hash: ContractHash,
        entry_point: &str,
        args: RuntimeArgs,
    ) -> Self {
        let args = Self::serialize_args(args);
        self.deploy_item.session_code = Some(ExecutableDeployItem::StoredContractByHash {
            hash,
            entry_point: entry_point.into(),
            args,
        });
        self
    }

    pub fn with_stored_session_named_key(
        mut self,
        name: &str,
        entry_point: &str,
        args: RuntimeArgs,
    ) -> Self {
        let args = Self::serialize_args(args);
        self.deploy_item.session_code = Some(ExecutableDeployItem::StoredContractByName {
            name: name.to_owned(),
            entry_point: entry_point.into(),
            args,
        });
        self
    }

    pub fn with_stored_versioned_contract_by_name(
        mut self,
        name: &str,
        version: Option<ContractVersion>,
        entry_point: &str,
        args: RuntimeArgs,
    ) -> Self {
        self.deploy_item.session_code = Some(ExecutableDeployItem::StoredVersionedContractByName {
            name: name.to_owned(),
            version,
            entry_point: entry_point.to_owned(),
            args: args.to_bytes().expect("should serialize runtime args"),
        });
        self
    }

    pub fn with_stored_versioned_contract_by_hash(
        mut self,
        hash: HashAddr,
        version: Option<ContractVersion>,
        entry_point: &str,
        args: RuntimeArgs,
    ) -> Self {
        self.deploy_item.session_code = Some(ExecutableDeployItem::StoredVersionedContractByHash {
            hash,
            version,
            entry_point: entry_point.to_owned(),
            args: args.to_bytes().expect("should serialize runtime args"),
        });
        self
    }

    pub fn with_stored_versioned_payment_contract_by_name(
        mut self,
        key_name: &str,
        version: Option<ContractVersion>,
        entry_point: &str,
        args: RuntimeArgs,
    ) -> Self {
        self.deploy_item.payment_code = Some(ExecutableDeployItem::StoredVersionedContractByName {
            name: key_name.to_owned(),
            version,
            entry_point: entry_point.to_owned(),
            args: args.to_bytes().expect("should serialize runtime args"),
        });
        self
    }

    pub fn with_stored_versioned_payment_contract_by_hash(
        mut self,
        hash: HashAddr,
        version: Option<ContractVersion>,
        entry_point: &str,
        args: RuntimeArgs,
    ) -> Self {
        self.deploy_item.payment_code = Some(ExecutableDeployItem::StoredVersionedContractByHash {
            hash,
            version,
            entry_point: entry_point.to_owned(),
            args: args.to_bytes().expect("should serialize runtime args"),
        });
        self
    }

    pub fn with_authorization_keys<T: Clone + Into<AccountHash>>(
        mut self,
        authorization_keys: &[T],
    ) -> Self {
        self.deploy_item.authorization_keys = authorization_keys
            .iter()
            .cloned()
            .map(|v| v.into())
            .collect();
        self
    }

    pub fn with_gas_price(mut self, gas_price: u64) -> Self {
        self.deploy_item.gas_price = gas_price;
        self
    }

    pub fn with_deploy_hash(mut self, hash: [u8; 32]) -> Self {
        self.deploy_item.deploy_hash = hash;
        self
    }

    pub fn build(self) -> DeployItem {
        DeployItem {
            address: self
                .deploy_item
                .address
                .unwrap_or_else(|| AccountHash::new([0u8; 32])),
            session: self
                .deploy_item
                .session_code
                .expect("should have session code"),
            payment: self
                .deploy_item
                .payment_code
                .expect("should have payment code"),
            gas_price: self.deploy_item.gas_price,
            authorization_keys: self.deploy_item.authorization_keys,
            deploy_hash: self.deploy_item.deploy_hash,
        }
    }

    fn serialize_args(args: RuntimeArgs) -> Vec<u8> {
        args.into_bytes().expect("should serialize args")
    }
}

impl Default for DeployItemBuilder {
    fn default() -> Self {
        let mut deploy_item: DeployItemData = Default::default();
        deploy_item.gas_price = 1;
        DeployItemBuilder { deploy_item }
    }
}
