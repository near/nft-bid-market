use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::AccountId;
use near_sdk::{collections::UnorderedMap, env};

pub type ActionId = String;
pub type PermissionId = String;
pub type ContractId = AccountId;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractPermission {
    permission_id: PermissionId,
    contract_id: ContractId,
    action_id: Option<ActionId>,
}

pub trait ContractAutorize {
    fn is_allowed(&self, contract_id: &ContractId, action_id: Option<&ActionId>) -> bool;
    fn verify_allowed(&self, contract_id: &ContractId, action_id: Option<&ActionId>);
    fn grant(&mut self, contract_id: ContractId, action_id: Option<ActionId>) -> bool;
    fn deny(&mut self, contract_id: ContractId, action_id: Option<&ActionId>) -> bool;
    fn grant_all(&mut self);
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractAuthorization {
    enabled: bool,
    granted_contracts: UnorderedMap<PermissionId, ContractPermission>,
}

impl ContractAuthorization {
    pub fn new(
        enabled: bool,
        granted_contracts: UnorderedMap<PermissionId, ContractPermission>,
    ) -> Self {
        Self {
            enabled,
            granted_contracts,
        }
    }
}

impl ContractAutorize for ContractAuthorization {
    fn is_allowed(&self, contract_id: &ContractId, action_id: Option<&ActionId>) -> bool {
        if self.enabled == false {
            return true;
        }

        let mut key: PermissionId = String::from(contract_id.clone());
        if action_id.is_some() {
            key = format!("{}{}", key, action_id.unwrap().clone());
        }
        return self.granted_contracts.get(&key).is_some();
    }

    fn panic_if_not_allowed(&self, contract_id: &ContractId, action_id: Option<&ActionId>) {
        if !self.is_allowed(contract_id, action_id) {
            env::panic_str(format!("Access to \"{}\" denied for this contract",action_id.clone()));
        }
    }

    fn grant(&mut self, contract_id: ContractId, action_id: Option<ActionId>) -> bool {
        let mut key: PermissionId = contract_id.to_string();
        if action_id.is_some() {
            key = format!("{}{}", key, action_id.as_ref().unwrap().clone());
        };
        self.granted_contracts
            .insert(
                &key,
                &ContractPermission {
                    permission_id: key.clone(),
                    contract_id: contract_id,
                    action_id: action_id,
                },
            )
            .is_none()
    }

    fn deny(&mut self, contract_id: ContractId, action_id: Option<&ActionId>) -> bool {
        let mut key: PermissionId = contract_id.to_string();
        if action_id.is_some() {
            key = format!("{}{}", key, action_id.unwrap());
        }
        self.granted_contracts.remove(&key).is_some()
    }

    fn grant_all(&mut self) {
        self.enabled = false;
    }
}