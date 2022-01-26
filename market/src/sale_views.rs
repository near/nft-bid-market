use crate::*;
use crate::common::*;

use std::cmp::min;
use crate::sale::{DELIMETER, ContractAndTokenId, SaleJson};

#[near_bindgen]
impl Market {

    /// views
    pub fn get_supply_sales(
        &self,
    ) -> U64 {
        U64(self.market.sales.len())
    }
    
    pub fn get_supply_by_owner_id(
        &self,
        account_id: AccountId,
    ) -> U64 {
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
    ) -> Vec<Sale> {
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
            tmp.push(self.market.sales.get(&keys.get(i).unwrap()).unwrap());
        }
        tmp
    }

    pub fn get_supply_by_nft_contract_id(
        &self,
        nft_contract_id: AccountId,
    ) -> U64 {
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
                .get(&format!("{}{}{}", &nft_contract_id, DELIMETER, &keys.get(i).unwrap()))
                .unwrap();
            let sale_json = self.json_from_sale(sale);
            tmp.push(sale_json);
        }
        tmp
    }

    pub fn get_supply_by_nft_token_type(
        &self,
        token_type: String,
    ) -> U64 {
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
            let sale = self
                .market
                .sales
                .get(&keys.get(i).unwrap())
                .unwrap();
            let sale_json = self.json_from_sale(sale);
            tmp.push(sale_json);
        }
        tmp
    }

    pub fn get_sale(&self, nft_contract_token: ContractAndTokenId) -> Option<SaleJson> {
        if let Some(sale) = self.market.sales.get(&nft_contract_token) {
            Some(self.json_from_sale(sale))
        } else {
            None
        }
    }
    
    fn json_from_sale(&self, sale: Sale) -> SaleJson {
        SaleJson {
            owner_id: sale.owner_id,
            nft_contract_id: sale.nft_contract_id,
            token_id: sale.token_id,
            sale_conditions: sale.sale_conditions,
            bids: sale.bids,
            created_at: sale.created_at,
            token_type: sale.token_type,

            start: sale.start,
            end: sale.end,
        }
    }
}
