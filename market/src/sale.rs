use std::collections::HashMap;

use near_sdk::{promise_result_as_success, Gas, PromiseResult};
use near_sdk::ext_contract;

use crate::*;
use common::*;

use bid::Bids;
pub type TokenSeriesId = String;

pub const GAS_FOR_FT_TRANSFER: Gas = Gas(5_000_000_000_000);
pub const GAS_FOR_ROYALTIES: Gas = Gas(115_000_000_000_000);
pub const GAS_FOR_NFT_TRANSFER: Gas = Gas(15_000_000_000_000);
pub const GAS_FOR_MINT: Gas = Gas(20_000_000_000_000);
//const BID_HISTORY_LENGTH_DEFAULT: u8 = 1;
const NO_DEPOSIT: Balance = 0;
pub static DELIMETER: &str = "||";


pub type SaleConditions = HashMap<FungibleTokenId, u128>;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

pub type ContractAndTokenId = String;
pub type FungibleTokenId = AccountId;
pub type TokenType = Option<String>;
pub type ContractAndSeriesId = String;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Sale {
    pub owner_id: AccountId,
    pub approval_id: u64,
    pub nft_contract_id: String,
    pub token_id: String,
    pub sale_conditions: SaleConditions,
    pub bids: Bids,
    pub created_at: u64,
    pub is_auction: Option<bool>,
    pub token_type: Option<String>,

    pub start: Option<u64>,
    pub end: Option<u64>,
}

impl Sale {
    pub fn in_limits(&self) -> bool {
        let mut res = true;
        let now = env::block_timestamp();
        if let Some(start) = self.start{
            res &= start < now;
        }
        if let Some(end) = self.end{
            res &= now < end;
        }
        res
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PurchaseArgs {
    pub nft_contract_id: AccountId,
    pub token_id: TokenId,
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MarketSales {
    pub owner_id: AccountId,
    pub sales: UnorderedMap<ContractAndTokenId, Sale>,
    pub token_series: UnorderedMap<ContractAndSeriesId, Balance>,
    pub by_owner_id: LookupMap<AccountId, UnorderedSet<ContractAndTokenId>>,
    pub by_nft_contract_id: LookupMap<AccountId, UnorderedSet<TokenId>>,
    pub by_nft_token_type: LookupMap<AccountId, UnorderedSet<ContractAndTokenId>>,
    pub ft_token_ids: UnorderedSet<FungibleTokenId>,
    pub storage_deposits: LookupMap<AccountId, Balance>,
    pub bid_history_length: u8,
}


#[near_bindgen]
impl Market {

    /// TODO remove without redirect to wallet? panic reverts
    #[payable]
    pub fn remove_sale(&mut self, nft_contract_id: AccountId, token_id: String) {
        assert_one_yocto();
        let sale = self.internal_remove_sale(nft_contract_id, token_id);
        let owner_id = env::predecessor_account_id();
        assert_eq!(owner_id, sale.owner_id, "Must be sale owner");
        self.refund_all_bids(&sale.bids);
    }

    #[payable]
    pub fn update_price(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        ft_token_id: FungibleTokenId,
        price: U128,
    ) {
        assert_one_yocto();
        let contract_id: AccountId = nft_contract_id;
        let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
        let mut sale = self.market.sales.get(&contract_and_token_id).expect("No sale");
        assert_eq!(
            env::predecessor_account_id(),
            sale.owner_id,
            "Must be sale owner"
        );
        if !self.market.ft_token_ids.contains(&ft_token_id) {
            env::panic_str(&format!("Token '{}' is not supported by this market", ft_token_id));
        }
        sale.sale_conditions.insert(ft_token_id, price.0);
        self.market.sales.insert(&contract_and_token_id, &sale);
    }

    #[payable]
    pub fn offer(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        start: Option<U64>,
        end: Option<U64>
    ) {
        let contract_id: AccountId = nft_contract_id;
        let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
        let mut sale = self.market.sales.get(&contract_and_token_id).expect("No sale");
        // Check that the sale is in progress
        require!(sale.in_limits(), "Either the sale is finished or it hasn't started yet");

        let buyer_id = env::predecessor_account_id();
        assert_ne!(sale.owner_id, buyer_id, "Cannot bid on your own sale.");
        let ft_token_id = "near".to_string();
        let price = *sale
            .sale_conditions
            .get(&ft_token_id.parse().unwrap())
            .expect("Not for sale in NEAR");

        let deposit = env::attached_deposit();
        assert!(deposit > 0, "Attached deposit must be greater than 0");

        if !sale.is_auction.unwrap_or(false) && deposit == price {
            self.process_purchase(
                contract_id,
                token_id,
                ft_token_id.parse().unwrap(),
                U128(deposit),
                buyer_id,
            );
        } else {
            if sale.is_auction.unwrap() && price > 0 {
                assert!(deposit >= price, "Attached deposit must be greater than reserve price");
            }
            self.add_bid(
                contract_and_token_id,
                deposit,
                ft_token_id.parse().unwrap(),
                buyer_id,
                &mut sale,
                start,
                end
            );
        }
    }

    pub fn accept_offer(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        ft_token_id: AccountId,
    ) {
        let contract_id: AccountId = nft_contract_id;
        let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
        // Check that the sale is in progress and remove bid before proceeding to process purchase
        let mut sale = self.market.sales.get(&contract_and_token_id).expect("No sale");
        require!(sale.in_limits(), "Either the sale is finished or it hasn't started yet");
        let bids_for_token_id = sale.bids.remove(&ft_token_id).expect("No bids");
        let bid = &bids_for_token_id[bids_for_token_id.len()-1];
        require!(
            bid.in_limits(),
            "Out of time limit of the bid"
        );
        self.market.sales.insert(&contract_and_token_id, &sale);
        // panics at `self.internal_remove_sale` and reverts above if predecessor is not sale.owner_id
        self.process_purchase(
            contract_id,
            token_id,
            ft_token_id,
            bid.price,
            bid.owner_id.clone(),
        );
    }

    #[private]
    pub fn process_purchase(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        ft_token_id: AccountId,
        price: U128,
        buyer_id: AccountId,
    ) -> Promise {
        let sale = self.internal_remove_sale(nft_contract_id.clone(), token_id.clone());

        ext_contract::nft_transfer_payout(
            buyer_id.clone(),
            token_id,
            sale.approval_id,
            "payout from market".to_string(),
            price,
			10,
            nft_contract_id,
            1,
            GAS_FOR_NFT_TRANSFER,
        )
        .then(ext_self::resolve_purchase(
            ft_token_id,
            buyer_id,
            sale,
            price,
            env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_ROYALTIES,
        ))
    }

    /// self callback

    #[private]
    pub fn resolve_purchase(
        &mut self,
        ft_token_id: AccountId,
        buyer_id: AccountId,
        sale: Sale,
        price: U128,
    ) -> U128 {

        // checking for payout information
        let payout_option = promise_result_as_success().and_then(|value| {
            // None means a bad payout from bad NFT contract
            near_sdk::serde_json::from_slice::<Payout>(&value)
                .ok()
                .and_then(|payout| {
                    // gas to do 10 FT transfers (and definitely 10 NEAR transfers)
                    if payout.payout.len() + sale.bids.len() > 10 || payout.payout.is_empty() {
                        env::log_str("Cannot have more than 10 royalties and sale.bids refunds");
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
        let payout = if let Some(payout_option) = payout_option {
            payout_option
        } else {
            if ft_token_id == "near".parse().unwrap() {
                Promise::new(buyer_id).transfer(u128::from(price));
            }
            // leave function and return all FTs in ft_resolve_transfer
            return price;
        };
        // Going to payout everyone, first return all outstanding bids (accepted offer bid was already removed)
        self.refund_all_bids(&sale.bids);

        // NEAR payouts
        if ft_token_id == "near".parse().unwrap() {
            for (receiver_id, amount) in payout.payout {
                Promise::new(receiver_id).transfer(amount.0);
            }
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

    #[payable]
    pub fn buy_token_copy(
        &mut self,
        nft_contract_id: AccountId,
        series_id: TokenSeriesId,
        reciever_id: AccountId
    ) -> Promise{
        let contract_and_series: ContractAndSeriesId = format!("{}{}{}", nft_contract_id, DELIMETER, series_id);
        let price = self.market.token_series.get(&contract_and_series).expect("Token series not found");
        let balance = env::attached_deposit() - price;
        let payout = ext_contract::nft_payout(series_id.clone(), price.into(), 10, nft_contract_id.clone(), 0, GAS_FOR_NFT_TRANSFER);
        ext_contract::nft_mint(
            series_id.clone(),
            reciever_id, 
            nft_contract_id, 
            balance,
            GAS_FOR_MINT).
            then(ext_self::resolve_mint(
                    env::predecessor_account_id(), 
                    series_id, 
                 env::attached_deposit().into(), 
env::current_account_id(), 
0,
GAS_FOR_MINT))
    }

    #[private]
    pub fn resolve_mint(
        &mut self,
        buyer_id: AccountId,
        token_series: TokenSeriesId,
        deposit: U128
    ) -> bool {
        require!(env::promise_results_count() == 1, "Contract expected a result on the callback");
        match env::promise_result(0) {
            PromiseResult::Successful(_) => true,
            _ => {Promise::new(buyer_id).transfer(deposit.into()); false},
        }
    }
}

/// self call

#[ext_contract(ext_self)]
trait ExtSelf {
    fn resolve_purchase(
        &mut self,
        ft_token_id: AccountId,
        buyer_id: AccountId,
        sale: Sale,
        price: U128,
    ) -> Promise;

    fn resolve_mint(
        &mut self,
        buyer_id: AccountId,
        token_series: TokenSeriesId,
        price: U128
    ) -> Promise;
}

/// external contract calls

#[ext_contract(ext_contract)]
trait ExtContract {
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: u64,
        memo: String,
        balance: U128,
		max_len_payout: u32,
    );
    fn ft_transfer(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>
    );
    fn nft_mint(
        &mut self,
        token_series_id: TokenSeriesId, 
        reciever_id: AccountId
    );
    fn nft_payout(
        &self, 
        token_id: String, 
        balance: U128,
        max_len_payout: u32
    ) -> Payout;
}
