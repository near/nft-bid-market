//use crate::*;
//use common::*;

pub const PAYOUT_TOTAL_VALUE:u128 = 10_000;
pub const PROTOCOL_FEE: u128 = 300; // 10_000 is 100%, so 300 is 3%

pub fn with_fees(price: u128) -> u128 {
    price * (PAYOUT_TOTAL_VALUE + PROTOCOL_FEE) / PAYOUT_TOTAL_VALUE
}

pub fn get_fee(price: u128) -> u128 {
    price * PROTOCOL_FEE / PAYOUT_TOTAL_VALUE
}
// #[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
// //#[serde(crate = "near_sdk::serde")]
// pub struct Fees {
//     pub protocol_fee: u128,
//     pub origins: UnorderedMap<AccountId, u128>,
//     pub royalty: u128,
// }

// impl Fees {
//     //Should be called in add_bid to check that the buyer attached enough deposit to pay the price + fee.
//     pub fn total_amount_fee_side(&self, price: U128) -> U128 {
//         U128(price.0 + self.calculate_protocol_fee(price).0 + self.calculate_origin_fee(price).0)
//     }

//     pub fn calculate_protocol_fee(&self, price: U128) -> U128 {
//         U128(price.0 * self.protocol_fee / 10_000 as u128)
//     }

//     pub fn calculate_origin_fee(&self, price: U128) -> U128 {
//         //    let accounts_and_fees = self.origins.get(&token).unwrap();
//         //    let mut total_origin: u128 = 0;
//         //    for (_account, fee) in accounts_and_fees.iter() {
//         //        total_origin += fee;
//         //    }
//         //    U128(price.0*total_origin)

//         let mut total_origin: u128 = 0;
//         for (_account, fee) in self.origins.iter() {
//             total_origin += fee;
//         }

//         U128(price.0 * total_origin / 10_000 as u128)
//     }

//     pub fn calculate_royalty(&self, price: U128) -> U128 {
//         U128(price.0 * self.royalty / 10_000 as u128)
//     }
// }

// //Fee side here is the account which buys nft. It pays with NEAR (or FT?).
// //It pays protocol_fees and origins.
// //Non-fee side pays protocol_fees, origins and royalty.

// //doTransfersWithFees on the fee side
// //transferPayouts on the non-fee side
