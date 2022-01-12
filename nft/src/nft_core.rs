use crate::*;
use near_contract_standards::non_fungible_token::{core::NonFungibleTokenCore, Token};

#[near_bindgen]
impl NonFungibleTokenCore for Nft {
    fn nft_transfer(
        &mut self,
        receiver_id: near_sdk::AccountId,
        token_id: near_contract_standards::non_fungible_token::TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        self.tokens
            .nft_transfer(receiver_id, token_id, approval_id, memo);
    }

    fn nft_transfer_call(
        &mut self,
        receiver_id: near_sdk::AccountId,
        token_id: near_contract_standards::non_fungible_token::TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> near_sdk::PromiseOrValue<bool> {
        self.tokens
            .nft_transfer_call(receiver_id, token_id, approval_id, memo, msg)
    }

    fn nft_token(
        &self,
        token_id: near_contract_standards::non_fungible_token::TokenId,
    ) -> Option<Token> {
        let owner_id = self.tokens.owner_by_id.get(&token_id)?;
        let mut token_id_iter = token_id.split(TOKEN_DELIMETER);
        let token_series_id = token_id_iter.next().unwrap().parse().unwrap();
        let mut series_metadata = self
            .token_series_by_id
            .get(&token_series_id)
            .unwrap()
            .metadata;
        let token_metadata = self
            .tokens
            .token_metadata_by_id
            .as_ref()
            .unwrap()
            .get(&token_id)
            .unwrap();
        let approved_account_ids = self
            .tokens
            .approvals_by_id
            .as_ref()
            .and_then(|by_id| by_id.get(&token_id).or_else(|| Some(HashMap::new())));
        series_metadata.issued_at = token_metadata.issued_at;
        series_metadata.copies = None;
        Some(Token {
            token_id,
            owner_id,
            metadata: Some(series_metadata),
            approved_account_ids,
        })
    }
}
