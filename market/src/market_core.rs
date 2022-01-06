use near_contract_standards::non_fungible_token::approval::NonFungibleTokenApprovalReceiver;
use crate::*;

#[near_bindgen]
impl NonFungibleTokenApprovalReceiver for Market {
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    ) -> near_sdk::PromiseOrValue<String> {
        require!(
            self.non_fungible_token_account_ids.contains(&env::predecessor_account_id()),
            "Only supports the one non-fungible token contract"
        );
        match msg.as_str() {
            _ => ()
        }
        todo!()
    }
}