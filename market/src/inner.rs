use std::convert::TryInto;

use crate::Market;
use crate::common::*;
use crate::sale::DELIMETER;
use crate::sale::Sale;

impl Market{

    pub(crate) fn internal_remove_sale(
        &mut self,
        nft_contract_id: AccountId,
        token_id: TokenId,
    ) -> Sale {
        let contract_and_token_id = format!("{}{}{}", &nft_contract_id, DELIMETER, token_id);
        let sale = self.market.sales.remove(&contract_and_token_id).expect("No sale");

        let mut by_owner_id = self.market.by_owner_id.get(&sale.owner_id).expect("No sale by_owner_id");
        by_owner_id.remove(&contract_and_token_id);
        if by_owner_id.is_empty() {
            self.market.by_owner_id.remove(&sale.owner_id);
        } else {
            self.market.by_owner_id.insert(&sale.owner_id, &by_owner_id);
        }

        let mut by_nft_contract_id = self
            .market
            .by_nft_contract_id
            .get(&nft_contract_id)
            .expect("No sale by nft_contract_id");
        by_nft_contract_id.remove(&token_id);
        if by_nft_contract_id.is_empty() {
            self.market.by_nft_contract_id.remove(&nft_contract_id);
        } else {
            self.market.by_nft_contract_id
                .insert(&nft_contract_id, &by_nft_contract_id);
        }

        // here AccountId is used as "token type", idk why so (adsick)
        let token_type = sale.token_type.clone();
        if let Some(token_type) = token_type {
            let mut by_nft_token_type = self.market.by_nft_token_type.get(&token_type.parse().unwrap()).expect("No sale by nft_token_type");
            by_nft_token_type.remove(&contract_and_token_id);
            if by_nft_token_type.is_empty() {
                self.market.by_nft_token_type.remove(&token_type.parse().unwrap());
            } else {
                self.market.by_nft_token_type.insert(&token_type.parse().unwrap(), &by_nft_token_type);
            }
        }

        sale
    }

}