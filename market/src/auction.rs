use crate::bid::{Bid, Bids};
use crate::market_core::AuctionArgs;
use crate::*;
use near_sdk::near_bindgen;
// should check calculation
pub const EXTENSION_DURATION: u64 = 15 * 60 * NANOS_PER_SEC; // 15 minutes
pub const MAX_DURATION: u64 = 1000 * 60 * 60 * 24 * NANOS_PER_SEC; // 1000 days

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Auction {
    pub owner_id: AccountId,
    pub approval_id: u64,
    pub nft_contract_id: AccountId,
    pub token_id: String,
    pub bid: Option<Bid>,
    pub created_at: u64, // do we need this for auctions?
    pub token_type: TokenType,
    pub minimal_step: u128,
    pub start_price: u128,
    pub buy_out_price: Option<u128>,

    pub start: u64,
    pub end: u64,
}

#[near_bindgen]
impl Market {
    fn start_auction(
        &mut self,
        args: AuctionArgs,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        nft_contract_id: AccountId,
    ) { // should return value
        
        require!(
            args.duration.0 >= EXTENSION_DURATION && args.duration.0 <= MAX_DURATION,
            "Incorrect duration"
        );
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
            token_type: args.token_type,
            minimal_step: args.minimal_step.into(),
            start_price: args.start_price.into(),
            buy_out_price: args.buy_out_price.map(|p| p.into()),
            start,
            end,
        };
        self.market.auctions.insert(&auction_id, &auction);
        self.market.next_auction_id += 1;
        env::log_str(&near_sdk::serde_json::to_string(&(auction_id, auction)).unwrap());
        //(auction_id, auction)
    }

    #[payable]
    pub fn put_bid(&mut self, auction_id: U128, token_type: TokenType) {
        let token_type = AccountId::new_unchecked(token_type.unwrap_or("near".to_owned()));
        require!(
            self.market.ft_token_ids.contains(&token_type),
            "token not supported"
        );
        let mut auction = self
            .market
            .auctions
            .get(&auction_id.into())
            .unwrap_or_else(|| env::panic_str("auction not active"));
        require!(env::block_timestamp() <= auction.end);
        let deposit = env::attached_deposit();
        if let Some(buy_out_price) = auction.buy_out_price {
            if buy_out_price == deposit {
                // TOOD: buyout
            }
        }
        let min_deposit = if let Some(bid) = auction.bid {
            bid.price.into()
        } else {
            auction.start_price
        };
        require!(deposit >= min_deposit);
        let bid = Bid {
            owner_id: env::predecessor_account_id(),
            price: deposit.into(),
            start: None,
            end: None,
        };
        auction.bid = Some(bid);
        if auction.end - env::block_timestamp() < EXTENSION_DURATION {
            auction.end = env::block_timestamp() + EXTENSION_DURATION;
        }
        self.market.auctions.insert(&auction_id.into(), &auction);
    }
}
