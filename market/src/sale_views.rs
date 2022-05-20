use crate::common::*;
use crate::*;

use crate::sale::{SaleJson, DELIMETER};
use std::cmp::min;

#[near_bindgen]
impl Market {
    /// views
    pub fn get_supply_sales(&self) -> U64 {
        U64(self.market.sales.len())
    }

    pub fn get_sales(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<SaleJson> {
        let sales = &self.market.sales;
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        sales
            .values()
            .skip(start_index as usize)
            .take(limit)
            .map(|sale| self.json_from_sale(sale))
            .collect()
    }

    pub fn get_supply_by_owner_id(&self, account_id: AccountId) -> U64 {
        let by_owner_id = self.market.by_owner_id.get(&account_id);
        if let Some(by_owner_id) = by_owner_id {
            U64(by_owner_id.len())
        } else {
            U64(0)
        }
    }

    pub fn get_sales_by_owner_id(
        &self,
        account_id: AccountId,
        from_index: U64,
        limit: u64,
    ) -> Vec<SaleJson> {
        let mut tmp = vec![];
        let by_owner_id = self.market.by_owner_id.get(&account_id);
        let sales = if let Some(by_owner_id) = by_owner_id {
            by_owner_id
        } else {
            return vec![];
        };
        let keys = sales.as_vector();
        let start = u64::from(from_index);
        let end = min(start + limit, sales.len());
        for i in start..end {
            let sale = self.market.sales.get(&keys.get(i).unwrap()).unwrap();
            tmp.push(self.json_from_sale(sale));
        }
        tmp
    }

    pub fn get_supply_by_nft_contract_id(&self, nft_contract_id: AccountId) -> U64 {
        let by_nft_contract_id = self.market.by_nft_contract_id.get(&nft_contract_id);
        if let Some(by_nft_contract_id) = by_nft_contract_id {
            U64(by_nft_contract_id.len())
        } else {
            U64(0)
        }
    }

    pub fn get_sales_by_nft_contract_id(
        &self,
        nft_contract_id: AccountId,
        from_index: U64,
        limit: u64,
    ) -> Vec<SaleJson> {
        let mut tmp = vec![];
        let by_nft_contract_id = self.market.by_nft_contract_id.get(&nft_contract_id);
        let sales = if let Some(by_nft_contract_id) = by_nft_contract_id {
            by_nft_contract_id
        } else {
            return vec![];
        };
        let keys = sales.as_vector();
        let start = u64::from(from_index);
        let end = min(start + limit, sales.len());
        for i in start..end {
            let sale = self
                .market
                .sales
                .get(&format!(
                    "{}{}{}",
                    &nft_contract_id,
                    DELIMETER,
                    &keys.get(i).unwrap()
                ))
                .unwrap();
            let sale_json = self.json_from_sale(sale);
            tmp.push(sale_json);
        }
        tmp
    }

    pub fn get_supply_by_nft_token_type(&self, token_type: String) -> U64 {
        let by_nft_token_type = self.market.by_nft_token_type.get(&token_type);
        if let Some(by_nft_token_type) = by_nft_token_type {
            U64(by_nft_token_type.len())
        } else {
            U64(0)
        }
    }

    pub fn get_sales_by_nft_token_type(
        &self,
        token_type: String,
        from_index: U64,
        limit: u64,
    ) -> Vec<SaleJson> {
        let mut tmp = vec![];
        let by_nft_token_type = self.market.by_nft_token_type.get(&token_type);
        let sales = if let Some(by_nft_token_type) = by_nft_token_type {
            by_nft_token_type
        } else {
            return vec![];
        };
        let keys = sales.as_vector();
        let start = u64::from(from_index);
        let end = min(start + limit, sales.len());
        for i in start..end {
            let sale = self.market.sales.get(&keys.get(i).unwrap()).unwrap();
            let sale_json = self.json_from_sale(sale);
            tmp.push(sale_json);
        }
        tmp
    }

    pub fn get_sale(&self, nft_contract_id: AccountId, token_id: TokenId) -> Option<SaleJson> {
        let contract_and_token_id = format!("{}{}{}", &nft_contract_id, DELIMETER, token_id);
        self.market
            .sales
            .get(&contract_and_token_id)
            .map(|sale| self.json_from_sale(sale))
    }

    pub fn get_total_bid_balance(&self, ft_token_id: Option<AccountId>) -> Balance {
        let owner_id: AccountId = env::predecessor_account_id();
        let ft = match ft_token_id {
            Some(ft) => ft,
            None => "near".parse().unwrap(),
        };
        self.market
            .bid_accounts
            .get(&owner_id)
            .expect("No account")
            .total_balance
            .get(&ft)
            .expect("No token for this account")
    }

    pub fn get_bid_by_index(&self, bid_id: BidIndex) -> Bid {
        self.market.bids_by_index.get(&bid_id).expect("No bid with this id")
    }

    /*pub fn get_bids_by_contract_and_token(&self, contract_and_token_id: ContractAndTokenId) -> BidsForContractAndTokenIdJson {
        self.market.bids.get(&contract_and_token_id).expect("No bid with this id")
    }*/

    pub fn get_bids_by_account_on_token(&self, owner_id: Option<AccountId>) -> Vec<ContractAndTokenId> {
        let owner_id = owner_id.unwrap_or(env::predecessor_account_id());
        self.market.bids_by_owner.get(&owner_id).expect("No bid with this id").keys_as_vector().to_vec()
    }

    pub fn get_bids_id_by_account_on(&self, owner_id: Option<AccountId>) -> Vec<BidIndex> {
        let owner_id = owner_id.unwrap_or(env::predecessor_account_id());
        let mut vec = Vec::new();
        let lookup_map = &self
            .market
            .bids_by_owner;
        let unordered_map = lookup_map
            .get(&owner_id)
            .expect("No bid with this id");
        let iter = unordered_map.values_as_vector().iter();
        
        for bid in iter {
            vec.push(bid.2);
        }
        vec
    }

    pub(crate) fn json_from_sale(&self, sale: Sale) -> SaleJson {
        SaleJson {
            owner_id: sale.owner_id,
            nft_contract_id: sale.nft_contract_id,
            token_id: sale.token_id,
            sale_conditions: sale.sale_conditions,
            created_at: sale.created_at.into(),
            token_type: sale.token_type,

            start: sale.start.map(|s| s.into()),
            end: sale.end.map(|e| e.into()),
            origins: sale.origins,
        }
    }
}
