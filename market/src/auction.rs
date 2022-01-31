use crate::bid::{Bid, Origins};
use crate::fee::PAYOUT_TOTAL_VALUE;
use crate::market_core::AuctionArgs;
use crate::sale::{
    ext_contract, ext_self, Payout, GAS_FOR_FT_TRANSFER, GAS_FOR_NFT_TRANSFER, GAS_FOR_ROYALTIES,
    NO_DEPOSIT,
};
use crate::*;
use near_sdk::{near_bindgen, promise_result_as_success};
// should check calculation
pub const EXTENSION_DURATION: u64 = 15 * 60 * NANOS_PER_SEC; // 15 minutes
pub const MAX_DURATION: u64 = 1000 * 60 * 60 * 24 * NANOS_PER_SEC; // 1000 days

use std::collections::HashMap;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Auction {
    pub owner_id: AccountId,
    pub approval_id: u64,
    pub nft_contract_id: AccountId,
    pub token_id: String,
    pub bid: Option<Bid>,
    pub created_at: u64,
    pub ft_token_id: AccountId,
    pub minimal_step: u128,
    pub start_price: u128,
    pub buy_out_price: Option<u128>,

    pub start: u64,
    pub end: u64,

    pub origins: Origins,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AuctionJson {
    pub owner_id: AccountId,
    pub nft_contract_id: AccountId,
    pub token_id: String,
    pub bid: Option<Bid>,
    pub created_at: u64,
    pub ft_token_id: AccountId,
    pub minimal_step: u128,
    pub start_price: u128,
    pub buy_out_price: Option<u128>,

    pub start: u64,
    pub end: u64,
}

#[near_bindgen]
impl Market {
    // Called in nft_on_approve to create a new auction
    // Returns a pair of the auction_id and the auction itself
    pub(crate) fn start_auction(
        &mut self,
        args: AuctionArgs,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        nft_contract_id: AccountId,
    ) -> (u128, AuctionJson) {
        // should return value

        require!(
            args.duration.0 >= EXTENSION_DURATION && args.duration.0 <= MAX_DURATION,
            "Incorrect duration"
        );
        let ft_token_id = self.token_type_to_ft_token_type(args.token_type);
        let start = args
            .start
            .map(|s| s.into())
            .unwrap_or_else(env::block_timestamp);
        require!(start >= env::block_timestamp(), "incorrect start time");
        let end = start + args.duration.0;
        let auction_id = self.market.next_auction_id;
        let origins = args
            .origins
            //.map(|s| s.into())
            .unwrap_or_default();
        let auction = Auction {
            owner_id,
            approval_id,
            nft_contract_id,
            token_id,
            bid: None,
            created_at: env::block_timestamp(),
            ft_token_id,
            minimal_step: args.minimal_step.0 * (PAYOUT_TOTAL_VALUE + PROTOCOL_FEE) / PAYOUT_TOTAL_VALUE,
            start_price: args.start_price.0 * (PAYOUT_TOTAL_VALUE + PROTOCOL_FEE) / PAYOUT_TOTAL_VALUE,
            buy_out_price: args.buy_out_price.map(|p| p.0 * (PAYOUT_TOTAL_VALUE + PROTOCOL_FEE) / PAYOUT_TOTAL_VALUE),
            start,
            end,
            origins,
        };
        self.market.auctions.insert(&auction_id, &auction);
        self.market.next_auction_id += 1;

        let auction_json = self.json_from_auction(auction);

        // log or return here?
        // env::log_str(&near_sdk::serde_json::to_string(&(auction_id, auction)).unwrap());
        (auction_id, auction_json)
    }

    // Adds a bid to the corresponding auction
    // Supports buyout and time extension
    #[payable]
    pub fn auction_add_bid(
        &mut self,
        auction_id: U128,
        token_type: TokenType,
        origins: Option<Origins>
    ) {
        let ft_token_id = self.token_type_to_ft_token_type(token_type);
        require!(
            self.market.ft_token_ids.contains(&ft_token_id),
            "token not supported"
        );
        require!(
            self.check_auction_in_progress(auction_id),
            "Auction is not in progress"
        );
        let mut auction = self
            .market
            .auctions
            .get(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("auction not active"));
        let deposit = env::attached_deposit();
        let min_deposit = self.get_minimal_next_bid(auction_id).0;

        // Check that the bid is not smaller than the minimal allowed bid
        require!(
            deposit >= min_deposit,
            format!("Should bid at least {}", min_deposit)
        );
        // Create a bid
        let bid = Bid {
            owner_id: env::predecessor_account_id(),
            price: deposit.into(),
            start: None,
            end: None,
            origins: origins.unwrap_or(HashMap::new()),
        };
        //Return previous bid
        if let Some(previous_bid) = auction.bid {
            self.refund_bid(ft_token_id, &previous_bid);
        }
        // Extend the auction if the bid is added EXTENSION_DURATION (15 min) before the auction end
        auction.bid = Some(bid);
        if auction.end - env::block_timestamp() < EXTENSION_DURATION {
            auction.end = env::block_timestamp() + EXTENSION_DURATION;
        }
        // If the price is bigger than the buy_out_price, the auction end is set to the current time
        if let Some(buy_out_price) = auction.buy_out_price {
            if buy_out_price <= deposit {
                auction.end = env::block_timestamp();
            }
        }
        self.market.auctions.insert(&auction_id.into(), &auction);
    }

    // Cancels the auction if it doesn't have a bid yet
    // Can be called by the auction owner
    #[payable]
    pub fn cancel_auction(&mut self, auction_id: U128) {
        assert_one_yocto();
        let auction = self
            .market
            .auctions
            .get(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("Auction is not active"));
        require!(
            auction.owner_id == env::predecessor_account_id(),
            "Only the auction owner can cancel the auction"
        );
        require!(
            auction.bid.is_none(),
            "Can't cancel the auction after the first bid is made"
        );
        self.market.auctions.remove(&auction_id.into());
    }

    // Finishes the auction if it has reached its end
    // Can be called by anyone
    pub fn finish_auction(&mut self, auction_id: U128) -> Promise {
        let auction = self
            .market
            .auctions
            .get(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("Auction is not active"));
        require!(
            env::block_timestamp() > auction.end,
            "Auction can be finalized only after the end time"
        );
        let final_bid = auction
            .bid
            .unwrap_or_else(|| env::panic_str("Can finalize only if there is a bid"));
        self.market.auctions.remove(&auction_id.into());
        let protocol_fee = final_bid.price.0 * PROTOCOL_FEE / (PAYOUT_TOTAL_VALUE + PROTOCOL_FEE);
        let new_price = final_bid.price.0 - protocol_fee;
        ext_contract::nft_transfer_payout(
            final_bid.owner_id.clone(),
            auction.token_id.clone(),
            auction.approval_id,
            None,
            U128(new_price),
            10,
            auction.nft_contract_id.clone(),
            1,
            GAS_FOR_NFT_TRANSFER,
        )
        .then(ext_self::resolve_finish_auction(
            auction.ft_token_id,
            final_bid.owner_id.clone(),
            auction.owner_id,
            final_bid.price,
            env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_ROYALTIES,
        ))
    }

    // self callback
    // If transfer of token succeded - count fees and transfer payouts
    // If failed - refund price to buyer
    #[private]
    pub fn resolve_finish_auction(
        &mut self,
        ft_token_id: AccountId,
        buyer_id: AccountId,
        owner_id: AccountId,
        price: U128,
    ) -> U128 {
        let payout_option = promise_result_as_success().and_then(|value| {
            near_sdk::serde_json::from_slice::<Payout>(&value)
                .ok()
                .and_then(|payout| {
                    if payout.payout.len() + 1 > 10 || payout.payout.is_empty() {
                        env::log_str("Cannot have more than 10 payouts and sale.bids refunds");
                        None
                    } else {
                        let mut remainder = price.0;
                        for &value in payout.payout.values() {
                            remainder = remainder.checked_sub(value.0)?;
                        }
                        if remainder <= 1 {
                            Some(payout)
                        } else {
                            None
                        }
                    }
                })
        });
        // is payout option valid?
        let mut payout = if let Some(payout_option) = payout_option {
            payout_option
        } else {
            if ft_token_id == "near".parse().unwrap() {
                Promise::new(buyer_id).transfer(u128::from(price));
            }
            // leave function and return all FTs in ft_resolve_transfer
            return price;
        };
        // Protocol fees
        let protocol_fee = price.0 * PROTOCOL_FEE / (PAYOUT_TOTAL_VALUE + PROTOCOL_FEE);

        let mut owner_payout: u128 = payout
            .payout
            .remove(&owner_id)
            .unwrap_or_else(|| unreachable!())
            .into();
        owner_payout -= protocol_fee * 2;
        // NEAR payouts
        if ft_token_id == "near".parse().unwrap() {
            // Royalties
            for (receiver_id, amount) in payout.payout {
                Promise::new(receiver_id).transfer(amount.0);
                owner_payout -= amount.0;
            }
            // Payouts
            Promise::new(owner_id).transfer(owner_payout);
            // refund all FTs (won't be any)
            price
        } else {
            // FT payouts
            for (receiver_id, amount) in payout.payout {
                ext_contract::ft_transfer(
                    receiver_id,
                    amount,
                    None,
                    ft_token_id.clone(),
                    1,
                    GAS_FOR_FT_TRANSFER,
                );
            }
            // keep all FTs (already transferred for payouts)
            U128(0)
        }
    }

    fn token_type_to_ft_token_type(&self, token_type: TokenType) -> AccountId {
        let token_type = if let Some(token_type) = token_type {
            AccountId::new_unchecked(token_type)
        } else {
            AccountId::new_unchecked("near".to_owned())
        };
        require!(
            self.market.ft_token_ids.contains(&token_type),
            "token not supported"
        );
        token_type
    }

    pub fn json_from_auction(&self, auction: Auction) -> AuctionJson {
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
}
