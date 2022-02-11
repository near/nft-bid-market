use crate::*;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LookupMap,
    env, AccountId,
};

pub type ActionId = String;
pub type PermissionId = String;

const DELIMETER: char = ':';

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractPermission {
    permission_id: PermissionId,
    contract_id: AccountId,
    action_id: ActionId,
}

pub trait ContractAutorize {
    fn is_allowed(&self, contract_id: &AccountId, action_id: &str) -> bool;
    fn panic_if_not_allowed(&self, contract_id: &AccountId, action_id: &str);
    fn grant(&mut self, contract_id: AccountId, action_id: ActionId) -> bool;
    fn deny(&mut self, contract_id: AccountId, action_id: ActionId) -> bool;
    fn set_authorization(&mut self, enabled: bool);
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractAuthorization {
    enabled: bool,
    granted_contracts: LookupMap<PermissionId, ContractPermission>, // shouldn't be set?
}

impl ContractAuthorization {
    pub fn new(
        enabled: bool,
        granted_contracts: LookupMap<PermissionId, ContractPermission>,
    ) -> Self {
        Self {
            enabled,
            granted_contracts,
        }
    }
}

impl ContractAutorize for ContractAuthorization {
    fn is_allowed(&self, contract_id: &AccountId, action_id: &str) -> bool {
        if !self.enabled {
            return true;
        }

        let key = format!("{}{}{}", contract_id, DELIMETER, action_id);

        self.granted_contracts.get(&key).is_some()
    }

    fn panic_if_not_allowed(&self, contract_id: &AccountId, action_id: &str) {
        if !self.is_allowed(contract_id, action_id) {
            env::panic_str(&format!(
                "Access to \"{}\" denied for this contract",
                action_id
            ));
        }
    }

    fn grant(&mut self, contract_id: AccountId, action_id: ActionId) -> bool {
        let key = format!("{}{}{}", contract_id, DELIMETER, action_id,);
        self.granted_contracts
            .insert(
                &key,
                &ContractPermission {
                    permission_id: key.clone(),
                    contract_id,
                    action_id,
                },
            )
            .is_none()
    }

    fn deny(&mut self, contract_id: AccountId, action_id: ActionId) -> bool {
        let key = format!("{}{}{}", contract_id, DELIMETER, action_id);
        self.granted_contracts.remove(&key).is_some()
    }

    fn set_authorization(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}


#[near_bindgen]
impl Nft {
    pub fn is_allowed(&self, contract_id: AccountId, action_id: &str) -> bool {
        self.contract_authorization.is_allowed(&contract_id, action_id)
    }

    pub fn grant(&mut self, contract_id: AccountId, action_id: ActionId) -> bool {
        require!(env::predecessor_account_id() == self.tokens.owner_id, "only owner can grant");
        self.contract_authorization.grant(contract_id, action_id)
    }

    pub fn deny(&mut self, contract_id: AccountId, action_id: ActionId) -> bool {
        require!(env::predecessor_account_id() == self.tokens.owner_id, "only owner can grant");
        self.contract_authorization.deny(contract_id, action_id)
    }

    pub fn set_authorization(&mut self, enabled: bool) {
        require!(env::predecessor_account_id() == self.tokens.owner_id, "only owner can grant");
        self.contract_authorization.set_authorization(enabled);
    }
}
