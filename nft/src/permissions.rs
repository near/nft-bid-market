use crate::*;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, AccountId,
};

pub trait ContractAutorize {
    fn is_allowed(&self, contract_id: &AccountId) -> bool;
    fn panic_if_not_allowed(&self, contract_id: &AccountId);
    fn grant(&mut self, contract_id: AccountId) -> bool;
    fn deny(&mut self, contract_id: AccountId) -> bool;
    fn set_authorization(&mut self, enabled: bool);
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct PrivateMint {
    enabled: bool,
    private_minters: LookupSet<AccountId>,
}

impl PrivateMint {
    pub fn new(enabled: bool, private_minters: LookupSet<AccountId>) -> Self {
        Self {
            enabled,
            private_minters,
        }
    }
}

impl ContractAutorize for PrivateMint {
    fn is_allowed(&self, contract_id: &AccountId) -> bool {
        !self.enabled || self.private_minters.contains(contract_id)
    }

    fn panic_if_not_allowed(&self, contract_id: &AccountId) {
        if !self.is_allowed(contract_id) {
            env::panic_str("Access to mint is denied for this contract");
        }
    }

    fn grant(&mut self, contract_id: AccountId) -> bool {
        self.private_minters.insert(&contract_id)
    }

    fn deny(&mut self, contract_id: AccountId) -> bool {
        self.private_minters.remove(&contract_id)
    }

    fn set_authorization(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[near_bindgen]
impl Nft {
    pub fn is_allowed(&self, contract_id: AccountId) -> bool {
        self.private_mint.is_allowed(&contract_id)
    }

    pub fn grant(&mut self, contract_id: AccountId) -> bool {
        require!(
            env::predecessor_account_id() == self.tokens.owner_id,
            "only owner can grant"
        );
        self.private_mint.grant(contract_id)
    }

    pub fn deny(&mut self, contract_id: AccountId) -> bool {
        require!(
            env::predecessor_account_id() == self.tokens.owner_id,
            "only owner can deny"
        );
        self.private_mint.deny(contract_id)
    }

    pub fn set_private_minting(&mut self, enabled: bool) {
        require!(
            env::predecessor_account_id() == self.tokens.owner_id,
            "only owner can enable/disable private minting"
        );
        self.private_mint.set_authorization(enabled);
    }
}
