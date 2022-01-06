mod sale;
mod token;
mod market_core;

use crate::sale::MarketSales;

use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NFT_METADATA_SPEC,
};
use near_sdk::{require, AccountId, PanicOnDefault};

use near_contract_standards::non_fungible_token::{metadata::TokenMetadata, TokenId};
use near_contract_standards::non_fungible_token::{NonFungibleToken, Token, self};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, LookupSet, UnorderedMap, UnorderedSet};
use near_sdk::{env, near_bindgen, BorshStorageKey};
use std::convert::TryInto;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Market {
    non_fungible_token_account_ids: LookupSet<AccountId>,
    market: MarketSales,
}


#[near_bindgen]
impl Market {
    #[init]
    pub fn new(nft_ids: Vec<AccountId>) -> Self {
        let mut non_fungible_token_account_ids = LookupSet::new(b"n");
        non_fungible_token_account_ids.extend(nft_ids);
        let market = MarketSales {
            //owner_id: "owner".from_str(),
            owner_id: String::new().try_into().unwrap(),
            sales: UnorderedMap::new(b"s"),
            by_owner_id: LookupMap::new(b"o"),
            by_nft_contract_id: LookupMap::new(b"c"),
            by_nft_token_type: LookupMap::new(b"t"),
            ft_token_ids: UnorderedSet::new(b"f"),
            storage_deposits: LookupMap::new(b"d"),
            bid_history_length: 0,
        };
        Self {
            non_fungible_token_account_ids, 
            market
        }
    }
}
