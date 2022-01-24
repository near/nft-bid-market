use crate::bid::{Bid, Bids};
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Auction {
    pub owner_id: AccountId,
    #[serde(skip_deserializing)] // not sure about this
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
}

#[near_bindgen]
impl Market {
    pub (crate) fn start_auction(
        &mut self,
        args: AuctionArgs,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        nft_contract_id: AccountId,
    ) -> (u128, Auction) {
        // should return value

        require!(
            args.duration.0 >= EXTENSION_DURATION && args.duration.0 <= MAX_DURATION,
            "Incorrect duration"
        );
        let ft_token_id = self.token_type_to_ft_token_type(args.token_type);
        let start = args.start.0;
        require!(start >= env::block_timestamp(), "incorrect start time");
        let end = start + args.duration.0;
        let auction_id = self.market.next_auction_id;
        let auction = Auction {
            owner_id,
            approval_id,
            nft_contract_id,
            token_id,
            bid: None,
            created_at: env::block_timestamp(),
            ft_token_id,
            minimal_step: args.minimal_step.into(),
            start_price: args.start_price.into(),
            buy_out_price: args.buy_out_price.map(|p| p.into()),
            start,
            end,
        };
        self.market.auctions.insert(&auction_id, &auction);
        self.market.next_auction_id += 1;

        // log or return here?
        // env::log_str(&near_sdk::serde_json::to_string(&(auction_id, auction)).unwrap());
        (auction_id, auction)
    }

    #[payable]
    pub fn put_bid(&mut self, auction_id: U128, token_type: TokenType) {
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
        //require!(env::block_timestamp() <= auction.end); //Should check auction.start too. Used check_auction_in_progress instead
        let deposit = env::attached_deposit();
        let min_deposit = self.get_minimal_next_bid(auction_id).0;
        require!(deposit >= min_deposit);
        let bid = Bid {
            owner_id: env::predecessor_account_id(),
            price: deposit.into(),
            start: None,
            end: None,
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
            if buy_out_price >= deposit {
                auction.end = env::block_timestamp();
            }
        }
        self.market.auctions.insert(&auction_id.into(), &auction);
    }

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
        //need to call process_purchase?
        self.market.auctions.remove(&auction_id.into());
        ext_contract::nft_transfer_payout(
            final_bid.owner_id.clone(),
            auction.token_id.clone(),
            auction.approval_id,
            None,
            final_bid.price,
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
        let protocol_fee = price.0 * PROTOCOL_FEE / 10_000u128;

        let mut owner_payout: u128 = payout
            .payout
            .remove(&owner_id)
            .unwrap_or_else(|| unreachable!())
            .into();
        owner_payout -= protocol_fee;
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
}
