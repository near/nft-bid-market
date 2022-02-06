use std::collections::HashMap;

use near_sdk::assert_one_yocto;

use crate::fee::{calculate_actual_amount, calculate_origins};
use crate::sale::{
    ext_contract, ContractAndTokenId, FungibleTokenId, Sale, DELIMETER, GAS_FOR_FT_TRANSFER,
};
use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Bid {
    pub owner_id: AccountId,
    pub price: U128,

    pub start: Option<U64>,
    pub end: Option<U64>,

    pub origins: Origins,
}

impl Bid {
    pub fn in_limits(&self) -> bool {
        let mut res = true;
        let now = env::block_timestamp();
        if let Some(start) = self.start {
            res &= start.0 < now;
        }
        if let Some(end) = self.end {
            res &= now < end.0;
        }
        res
    }
}

pub type Bids = HashMap<FungibleTokenId, Vec<Bid>>;
pub type Origins = HashMap<AccountId, u32>;

#[near_bindgen]
impl Market {
    // Adds a bid if it is higher than the last bid of this ft_token_id
    // Refunds the previous bid (of this ft_token_id)
    #[allow(clippy::too_many_arguments)]
    #[private]
    pub fn add_bid(
        &mut self,
        contract_and_token_id: ContractAndTokenId,
        amount: Balance,
        ft_token_id: AccountId,
        buyer_id: AccountId,
        sale: &mut Sale,
        start: Option<U64>,
        end: Option<U64>,
        origins: Option<Origins>,
    ) {
        require!(
            self.market.ft_token_ids.contains(&ft_token_id),
            format!("Token {} not supported by this market", ft_token_id)
        );
        let total_origins = if let Some(ref origins) = origins {
            calculate_origins(origins)
        } else {
            0
        };

        require!(total_origins < 4_700); // TODO: FINDOUT MAX ORIGINS
        let actual_amount = calculate_actual_amount(amount, total_origins);

        // store a bid and refund any current bid lower
        let new_bid = Bid {
            owner_id: buyer_id,
            price: U128(amount),
            start,
            end,
            origins: origins.unwrap_or_default(),
        };

        let bids_for_token_id = sale
            .bids
            .entry(ft_token_id.clone())
            .or_insert_with(Vec::new);
        if let Some(current_bid) = bids_for_token_id.last() {
            let current_origins = calculate_origins(&current_bid.origins);
            let current_amount = calculate_actual_amount(current_bid.price.0, current_origins);
            require!(
                actual_amount > current_amount,
                format!(
                    "Can't pay less than or equal to current bid price: {}",
                    current_bid.price.0
                )
            );
        }

        bids_for_token_id.push(new_bid);
        if bids_for_token_id.len() > self.market.bid_history_length as usize {
            // Need to refund the earliest bid before removing it
            let early_bid = &bids_for_token_id[0];
            self.refund_bid(ft_token_id, early_bid);
            bids_for_token_id.remove(0);
        }

        self.market.sales.insert(&contract_and_token_id, sale);
    }

    #[payable]
    pub fn remove_bid(
        &mut self,
        nft_contract_id: AccountId,
        token_id: TokenId,
        ft_token_id: AccountId,
        bid: Bid,
    ) {
        assert_one_yocto();
        assert_eq!(
            env::predecessor_account_id(),
            bid.owner_id,
            "Must be bid owner"
        );
        self.internal_remove_bid(nft_contract_id, &ft_token_id, token_id, &bid);
        self.refund_bid(ft_token_id, &bid);
    }

    // Cancels the bid if it has ended
    // Refunds it
    pub fn cancel_bid(
        &mut self,
        nft_contract_id: AccountId,
        token_id: TokenId,
        ft_token_id: AccountId,
        bid: Bid,
    ) {
        if let Some(end) = bid.end {
            let is_finished = env::block_timestamp() >= end.0;
            require!(is_finished, "The bid hasn't ended yet");
        }
        self.internal_remove_bid(nft_contract_id, &ft_token_id, token_id, &bid);
        self.refund_bid(ft_token_id, &bid);
    }
    /*
    // Cancel all expired bids
    pub fn cancel_expired_bids(
        &mut self,
        nft_contract_id: AccountId,
        token_id: TokenId,
        ft_token_id: AccountId,
    ) {
        let contract_and_token_id = format!("{}{}{}", &nft_contract_id, DELIMETER, token_id);
        /*let mut sale = self
            .market
            .sales
            .get(&contract_and_token_id)
            .expect("No sale");*/
        let mut sale = self
            .market
            .sales
            .get(&contract_and_token_id)
            .expect("No sale");
        let bid_vec = self
            .market
            .sales
            .get(&contract_and_token_id)
            .expect("No sale")
            .bids
            .get_mut(&ft_token_id)
            .expect("No token");
        bid_vec.retain(|bid_from_vec| {
            let mut not_finished = true;
            if let Some(end) = bid_from_vec.end {
                //is_finished &= env::block_timestamp() >= end.0;
                if env::block_timestamp() >= end.0 {
                    self.refund_bid(ft_token_id.clone(), bid_from_vec);
                    not_finished = false;
                };
            }
            not_finished
        });
        if bid_vec.len() == 1 {
            //If there is no bids left, should remove ft_token_id from the HashMap
            sale.bids.remove(&ft_token_id);
        } else {
            sale.bids.insert(ft_token_id.clone(),  bid_vec.to_vec());
            self.market.sales.insert(&contract_and_token_id, &sale);
        };
        /*for (index, bid_from_vec) in bid_vec.iter().rev().enumerate() {
            // Check whether the bid has ended
            let mut is_finished = false;
            if let Some(end) = bid_from_vec.end {
                //is_finished &= env::block_timestamp() >= end.0;
                if env::block_timestamp() >= end.0 {
                    is_finished = true;
                };
            }
            if is_finished {
                env::log_str("IS_FINISHED");
                if bid_vec.len() == 1 {
                    //If the vector contained only one bid, should remove ft_token_id from the HashMap
                    sale.bids.remove(&ft_token_id);
                } else {
                    //If there are several bids for this ft_token_id, should remove one bid
                    sale.bids
                        .get_mut(&ft_token_id)
                        .expect("No token")
                        .remove(index);
                };
                self.market.sales.insert(&contract_and_token_id, &sale);

                self.refund_bid(ft_token_id.clone(), bid_from_vec);
            } else {
                env::log_str(format!("NOT_FINISHED, now: {}", env::block_timestamp()).as_str());
            };
        } */
    }*/
}

impl Market {
    pub(crate) fn refund_all_bids(&mut self, bids_map: &Bids) {
        for (ft, bids) in bids_map {
            for bid in bids {
                self.refund_bid((*ft).clone(), bid);
            }
        }
    }

    pub(crate) fn refund_bid(&mut self, bid_ft: FungibleTokenId, bid: &Bid) {
        if bid_ft.as_str() == "near" {
            Promise::new(bid.owner_id.clone()).transfer(u128::from(bid.price));
        } else {
            ext_contract::ft_transfer(
                bid.owner_id.clone(),
                bid.price,
                None,
                bid_ft,
                1,
                GAS_FOR_FT_TRANSFER,
            );
        }
    }
}
