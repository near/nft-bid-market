use crate::auction::{Auction, AuctionJson};
use crate::common::*;
use crate::*;

#[near_bindgen]
impl Market {
    pub fn get_current_buyer(&self, auction_id: U128) -> Option<AccountId> {
        let auction = self
            .market
            .auctions
            .get(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("Auction does not exist"));
        if let Some(bid) = auction.bid {
            Some(bid.owner_id)
        } else {
            None
        }
    }

    pub fn check_auction_in_progress(&self, auction_id: U128) -> bool {
        let auction = self
            .market
            .auctions
            .get(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("Auction does not exist"));
        auction.end >= env::block_timestamp() && auction.start < env::block_timestamp()
    }

    pub fn get_auction_json(&self, auction_id: U128) -> AuctionJson {
        let auction = self.market
            .auctions
            .get(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("Auction does not exist"));
        AuctionJson {
            owner_id: auction.owner_id,
            nft_contract_id: auction.nft_contract_id,
            token_id: auction.token_id,
            bid: auction.bid,
            created_at: auction.created_at,
            ft_token_id: auction.ft_token_id,
            minimal_step: auction.minimal_step,
            start_price: auction.start_price,
            buy_out_price: auction.buy_out_price,
            start: auction.start,
            end: auction.end,
        }
    }

    // Returns the minimum amount of the next auction bid (not including fees)
    pub fn get_minimal_next_bid(&self, auction_id: U128) -> U128 {
        let auction = self
            .market
            .auctions
            .get(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("Auction does not exist"));
        let min_deposit = if let Some(ref bid) = auction.bid {
            bid.price.0 + auction.minimal_step
        } else {
            auction.start_price
        };
        U128(min_deposit)
    }

    pub fn get_current_bid(&self, auction_id: U128) -> Option<U128> {
        let auction = self
            .market
            .auctions
            .get(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("Auction does not exist"));
        auction.bid.map(|bid| bid.price)
    }

    //pub fn get_bid_total_amount() -> U128;
}
