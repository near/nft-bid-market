mod sale;
mod token;
mod market_core;

use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NFT_METADATA_SPEC,
};
use near_sdk::{require, AccountId, PanicOnDefault};

use near_contract_standards::non_fungible_token::{metadata::TokenMetadata, TokenId};
use near_contract_standards::non_fungible_token::{NonFungibleToken, Token, self};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, LookupSet};
use near_sdk::{env, near_bindgen, BorshStorageKey};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Market {
    non_fungible_token_account_ids: LookupSet<AccountId>,
}


#[near_bindgen]
impl Market {
    #[init]
    pub fn new(nft_ids: Vec<AccountId>) -> Self {
        let mut non_fungible_token_account_ids = LookupSet::new(b"n");
        non_fungible_token_account_ids.extend(nft_ids);
        Self {
            non_fungible_token_account_ids
        }
    }
}
