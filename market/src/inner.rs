use crate::common::*;
use crate::bid::Bid;
use crate::sale::{Sale, DELIMETER};
use crate::Market;

impl Market {
    pub(crate) fn internal_remove_sale(
        &mut self,
        nft_contract_id: AccountId,
        token_id: TokenId,
    ) -> Sale {
        let contract_and_token_id = format!("{}{}{}", &nft_contract_id, DELIMETER, token_id);
        let sale = self
            .market
            .sales
            .remove(&contract_and_token_id)
            .expect("No sale");

        let mut by_owner_id = self
            .market
            .by_owner_id
            .get(&sale.owner_id)
            .expect("No sale by_owner_id");
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
            self.market
                .by_nft_contract_id
                .insert(&nft_contract_id, &by_nft_contract_id);
        }

        // here AccountId is used as "token type", idk why so (adsick)
        let token_type = sale.token_type.clone();
        if let Some(token_type) = token_type {
            let mut by_nft_token_type = self
                .market
                .by_nft_token_type
                .get(&token_type.parse().unwrap())
                .expect("No sale by nft_token_type");
            by_nft_token_type.remove(&contract_and_token_id);
            if by_nft_token_type.is_empty() {
                self.market
                    .by_nft_token_type
                    .remove(&token_type.parse().unwrap());
            } else {
                self.market
                    .by_nft_token_type
                    .insert(&token_type.parse().unwrap(), &by_nft_token_type);
            }
        }

        sale
    }

    pub(crate) fn internal_remove_bid(
        &mut self,
        nft_contract_id: AccountId,
        token_id: TokenId,
        bid: &Bid,
    ) -> Sale {
        let contract_and_token_id = format!("{}{}{}", &nft_contract_id, DELIMETER, token_id);
        let sale = self
            .market
            .sales
            .get(&contract_and_token_id)
            .expect("No sale");
        let bid_vec = sale.bids.get(&token_id.parse().unwrap()).expect("No token");

        let mut sale = self
            .market
            .sales
            .get(&contract_and_token_id) //тут мы по сути достаём значение из хранилища
            .expect("No sale");

        for (index, bid_from_vec) in bid_vec.iter().enumerate() {
            if bid_from_vec.owner_id == bid.owner_id && bid_from_vec.price == bid.price {
                sale.bids
                    .get_mut(&token_id.parse().unwrap())
                    .expect("No token")
                    .remove(index); // что-то с ним делаем

                // но его ещё надо записать обратно
                self.market.sales.insert(&contract_and_token_id, &sale);
            };
        }

        sale
    }
}
