use std::collections::BTreeSet;

use types::account::AccountHash;

use crate::{engine_state::executable_deploy_item::ExecutableDeployItem, DeployHash};

type GasPrice = u64;

/// Represents a deploy to be executed.  Corresponds to the similarly-named ipc protobuf message.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DeployItem {
    pub address: AccountHash,
    pub session: ExecutableDeployItem,
    pub payment: ExecutableDeployItem,
    pub gas_price: GasPrice,
    pub authorization_keys: BTreeSet<AccountHash>,
    pub deploy_hash: DeployHash,
}

impl DeployItem {
    /// Creates a [`DeployItem`].
    pub fn new(
        address: AccountHash,
        session: ExecutableDeployItem,
        payment: ExecutableDeployItem,
        gas_price: GasPrice,
        authorization_keys: BTreeSet<AccountHash>,
        deploy_hash: DeployHash,
    ) -> Self {
        DeployItem {
            address,
            session,
            payment,
            gas_price,
            authorization_keys,
            deploy_hash,
        }
    }
}
