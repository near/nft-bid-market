mod bid;
mod sale;
mod token;
mod market_core;
mod common;
mod inner;
mod sale_views;
mod fee;

use common::*;

use crate::sale::{Sale, MarketSales, SaleConditions, TokenType, BID_HISTORY_LENGTH_DEFAULT};
use crate::fee::Fees;
use std::collections::HashMap;

const STORAGE_PER_SALE: u128 = 1000 * STORAGE_PRICE_PER_BYTE;
const PROTOCOL_FEE: u128 = 300; // 10_000 is 100%, so 300 is 3%
const ORIGIN: u128 = 300;

/// Helper structure to for keys of the persistent collections.
#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    Sales,
    ByOwnerId,
    TokenSeries,
    ByOwnerIdInner { account_id_hash: CryptoHash },
    ByNFTContractId,
    ByNFTContractIdInner { account_id_hash: CryptoHash },
    ByNFTTokenType,
    ByNFTTokenTypeInner { token_type_hash: CryptoHash },
    FTTokenIds,
    StorageDeposits,
    OriginFees,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Market {
    non_fungible_token_account_ids: LookupSet<AccountId>,
    market: MarketSales,
    //fees: Fees,
}


#[near_bindgen]
impl Market {
    #[init]
    pub fn new(nft_ids: Vec<AccountId>, owner_id: AccountId) -> Self {
        let mut non_fungible_token_account_ids = LookupSet::new(b"n");
        non_fungible_token_account_ids.extend(nft_ids);
        let mut tokens = UnorderedSet::new(StorageKey::FTTokenIds);
        tokens.insert(&AccountId::new_unchecked("near".to_owned()));
        let market = MarketSales {
            owner_id,
            series_sales: UnorderedMap::new(StorageKey::TokenSeries),
            sales: UnorderedMap::new(StorageKey::Sales),
            by_owner_id: LookupMap::new(StorageKey::ByOwnerId),
            by_nft_contract_id: LookupMap::new(StorageKey::ByNFTContractId),
            by_nft_token_type: LookupMap::new(StorageKey::ByNFTTokenType),
            ft_token_ids: tokens,
            storage_deposits: LookupMap::new(StorageKey::StorageDeposits),
            bid_history_length: BID_HISTORY_LENGTH_DEFAULT,
        };
        //let mut origins = LookupMap::new(StorageKey::OriginFees);
        //let mut origin = UnorderedMap::new(b'o').insert(&AccountId::new_unchecked("near".to_owned()), &ORIGIN);
        //origins.insert("near", &origin);
        //let fees = Fees {
        //    protocol_fee: PROTOCOL_FEE, 
        //    origins
        //};
        Self {
            non_fungible_token_account_ids, 
            market, 
        //    fees
        }
    }

    #[payable]
    pub fn storage_withdraw(&mut self) {
        assert_one_yocto();
        let owner_id = env::predecessor_account_id();
        let mut amount = self.market.storage_deposits.remove(&owner_id).unwrap_or(0);
        let sales = self.market.by_owner_id.get(&owner_id);
        let len = sales.map(|s| s.len()).unwrap_or_default();
        let diff = u128::from(len) * STORAGE_PER_SALE;
        amount -= diff;
        if amount > 0 {
            Promise::new(owner_id.clone()).transfer(amount);
        }
        if diff > 0 {
            self.market.storage_deposits.insert(&owner_id, &diff);
        }
    }

    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<AccountId>) {
        let storage_account_id = account_id
            .unwrap_or_else(env::predecessor_account_id);
        let deposit = env::attached_deposit();
        assert!(
            deposit >= STORAGE_PER_SALE,
            "Requires minimum deposit of {}",
            STORAGE_PER_SALE
        );
        let mut balance: u128 = self.market.storage_deposits.get(&storage_account_id).unwrap_or(0);
        balance += deposit;
        self.market.storage_deposits.insert(&storage_account_id, &balance);
    }

    pub fn storage_amount(&self) -> U128 {
        U128(STORAGE_PER_SALE)
    }
}
