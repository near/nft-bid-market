mod sale;
mod token;
mod market_core;
mod common;
mod inner;
mod sale_views;

use common::*;

use crate::sale::{Sale, MarketSales, SaleConditions, TokenType};
use std::collections::HashMap;

const STORAGE_PER_SALE: u128 = 1000 * STORAGE_PRICE_PER_BYTE;

/// Helper structure to for keys of the persistent collections.
#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    Sales,
    ByOwnerId,
    ByOwnerIdInner { account_id_hash: CryptoHash },
    ByNFTContractId,
    ByNFTContractIdInner { account_id_hash: CryptoHash },
    ByNFTTokenType,
    ByNFTTokenTypeInner { token_type_hash: CryptoHash },
    FTTokenIds,
    StorageDeposits,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Market {
    non_fungible_token_account_ids: LookupSet<AccountId>,
    market: MarketSales,
}


#[near_bindgen]
impl Market {
    #[init]
    pub fn new(nft_ids: Vec<AccountId>, owner_id: AccountId) -> Self {
        let mut non_fungible_token_account_ids = LookupSet::new(b"n");
        non_fungible_token_account_ids.extend(nft_ids);
        let market = MarketSales {
            owner_id,
            sales: UnorderedMap::new(StorageKey::Sales),
            by_owner_id: LookupMap::new(StorageKey::ByOwnerId),
            by_nft_contract_id: LookupMap::new(StorageKey::ByNFTContractId),
            by_nft_token_type: LookupMap::new(StorageKey::ByNFTTokenType),
            ft_token_ids: UnorderedSet::new(StorageKey::FTTokenIds),
            storage_deposits: LookupMap::new(StorageKey::StorageDeposits),
            bid_history_length: 1,
        };
        Self {
            non_fungible_token_account_ids, 
            market
        }
    }
}
