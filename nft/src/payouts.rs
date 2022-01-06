use std::collections::HashMap;

use near_contract_standards::non_fungible_token::core::NonFungibleTokenCore;
use near_sdk::{assert_one_yocto, promise_result_as_success};
use near_sdk::json_types::{ValidAccountId, U128, U64};

use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

pub trait Payouts {
    /// Given a `token_id` and NEAR-denominated balance, return the `Payout`.
    /// struct for the given token. Panic if the length of the payout exceeds
    /// `max_len_payout.`
    fn nft_payout(&self, token_id: String, balance: U128, max_len_payout: u32) -> Payout;
    /// Given a `token_id` and NEAR-denominated balance, transfer the token
    /// and return the `Payout` struct for the given token. Panic if the
    /// length of the payout exceeds `max_len_payout.`
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        approval_id: u64,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout;
}

#[near_bindgen]
impl Payouts for Market {
    fn nft_payout(&self, token_id: String, balance: U128, max_len_payout: u32) -> Payout {
        todo!()
    }
    #[payable]
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        approval_id: u64,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout {
        assert_one_yocto();
        let payout = self.nft_payout(token_id.clone(), balance, max_len_payout);
        self.nft_transfer(receiver_id, token_id, Some(approval_id), None);
        payout
    }
}
