use crate::*;
use common::*;

pub type FeeAmount = u128; //should be f64
pub type FeeAccountAndAmount = UnorderedMap<AccountId, FeeAmount>;

pub struct Fees {
    pub protocol_fee: FeeAmount,
    pub origins: UnorderedMap<TokenId, FeeAccountAndAmount>,
    pub royalty: FeeAmount,
}

//pub `fees: Fees` should be added to Sale

impl Fees {
    //Should be called in add_bid to check that the buyer attached enough deposit to pay the price + fee.
    pub fn total_amount_fee_side(&self, price: U128, token: TokenId) -> U128 {
        U128(self.calculate_protocol_fees(price).0 + self.calculate_origins(price, token).0)
    }

    //Should nft_on_approve be payable?
    //Should be payed by the token owner.
    pub fn total_amount_non_fee_side(&self, price: U128, token: TokenId) -> U128 {
        U128(self.calculate_protocol_fees(price).0 + self.calculate_royalties(price, token).0)
    }

    pub fn calculate_protocol_fees(&self, price: U128) -> U128 {
        U128(price.0*self.protocol_fee)
    }

    pub fn calculate_origins(&self, price: U128, token: TokenId) -> U128 {
        let accounts_and_fees = self.origins.get(&token).unwrap();
        let mut total_origin: u128 = 0;
        for (_account, fee) in accounts_and_fees.iter() {
            total_origin += fee;
        }
        U128(price.0*total_origin)
    }

    pub fn calculate_royalties(&self, price: U128, token: TokenId) -> U128 {
        U128(price.0*self.royalty)
    }
}

//Fee side here is the account which buys nft. It pays with NEAR (or FT?).
//It pays protocol_fees and origins.
//Non-fee side pays protocol_fees, origins and royalty.

//doTransfersWithFees on the fee side
//transferPayouts on the non-fee side
