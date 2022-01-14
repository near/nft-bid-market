use std::collections::HashMap;

use near_contract_standards::non_fungible_token::core::NonFungibleTokenCore;
use near_sdk::json_types::U128;

use crate::token_series::SeriesMintArgs;
use crate::*;

pub const ROYALTY_TOTAL_VALUE: u128 = 10_000;
pub const MAXIMUM_ROYALTY: u32 = 5_000;
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

fn assert_at_least_one_yocto() {
    require!(env::attached_deposit() >= 1, "Requires attached deposit of at least 1 yoctoNEAR")
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
        memo: Option<String>,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout;
}

#[near_bindgen]
impl Payouts for Nft {
    fn nft_payout(&self, token_id: String, balance: U128, max_len_payout: u32) -> Payout {
        let token_owner = self.tokens.owner_by_id.get(&token_id).expect("no token id");

        let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
        let token_series_id = token_id_iter.next().unwrap().to_owned();
        let royalty = self
            .token_series_by_id
            .get(&token_series_id)
            .expect("no type")
            .royalty;
        require!(royalty.len() as u32 <= max_len_payout, "Too many recievers");
        let mut total_payout = 0;
        let balance = Balance::from(balance);
        let mut payout: Payout = Payout {
            payout: HashMap::new(),
        };
        for (k, v) in royalty.iter() {
            if *k != token_owner {
                payout
                    .payout
                    .insert(k.clone(), royalty_to_payout(*v, balance));
                total_payout += v;
            }
        }
        require!(
            total_payout <= ROYALTY_TOTAL_VALUE as u32,
            "Royalty total value should be < 10000"
        );
        payout.payout.insert(
            token_owner,
            royalty_to_payout(ROYALTY_TOTAL_VALUE as u32 - total_payout, balance),
        );
        payout
    }
    #[payable]
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        approval_id: u64,
        memo: Option<String>,
        balance: U128,
        max_len_payout: u32,
    ) -> Payout {
        assert_at_least_one_yocto();

        if let Some(series_mint_args) = memo {
            let SeriesMintArgs {
                token_series_id,
                receiver_id,
            } = near_sdk::serde_json::from_str(&series_mint_args).expect("invalid SeriesMintArgs");
            self.nft_mint(token_series_id, receiver_id);
        } else {
            self.nft_transfer(receiver_id, token_id.clone(), Some(approval_id), None);
        }
        self.nft_payout(token_id, balance, max_len_payout)
    }
}

fn royalty_to_payout(a: u32, b: Balance) -> U128 {
    U128(a as u128 * b / ROYALTY_TOTAL_VALUE)
}
