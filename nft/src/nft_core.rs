use crate::*;
use near_contract_standards::non_fungible_token::{Token, core::NonFungibleTokenCore};

#[near_bindgen]
impl NonFungibleTokenCore for Nft {
    fn nft_transfer(
        &mut self,
        receiver_id: near_sdk::AccountId,
        token_id: near_contract_standards::non_fungible_token::TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        self.tokens.nft_transfer(receiver_id, token_id, approval_id, memo);
    }

    fn nft_transfer_call(
        &mut self,
        receiver_id: near_sdk::AccountId,
        token_id: near_contract_standards::non_fungible_token::TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> near_sdk::PromiseOrValue<bool> {
        self.tokens.nft_transfer_call(receiver_id, token_id, approval_id, memo, msg)
    }

    fn nft_token(&self, token_id: near_contract_standards::non_fungible_token::TokenId) -> Option<Token> {
        self.tokens.nft_token(token_id)
    }
}