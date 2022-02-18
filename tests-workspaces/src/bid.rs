use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::utils::{init_market, init_nft, mint_token, check_outcome_success, check_outcome_fail};
use near_units::{parse_gas, parse_near};

/*
Refunds a bid, removes it from the list
*/
#[tokio::test]
async fn remove_bid_positive() -> anyhow::Result<()> {
    Ok(())
}

/*
- Should panic unless 1 yoctoNEAR is attached
- Should panic if there is no sale with the given `nft_contract_id` and `token_id`
- Should panic if there is no bids with `ft_token_id`
*/
#[tokio::test]
async fn remove_bid_negative() -> anyhow::Result<()> {
    Ok(())
}

/*
Refunds a bid, removes it from the list
*/
#[tokio::test]
async fn cancel_bid_positive() -> anyhow::Result<()> {
    Ok(())
}

/*
- Should panic if the bid isn't finished yet
- Should panic if the bid doesn't have end time
- Should panic if there is no sale with the given `nft_contract_id` and `token_id`
- Should panic if there is no bids with `ft_token_id`
- Should panic if there is no bid with given `owner_id` and `price`
*/
#[tokio::test]
async fn cancel_bid_negative() -> anyhow::Result<()> {
    Ok(())
}

/*
- Refunds all expired bids, removes them from the list
*/
#[tokio::test]
async fn cancel_expired_bids_positive() -> anyhow::Result<()> {
    Ok(())
}

/*
- Should panic if there is no sale with the given `nft_contract_id` and `token_id`
- Should panic if there is no bids with `ft_token_id`
*/
#[tokio::test]
async fn cancel_expired_bids_negative() -> anyhow::Result<()> {
    Ok(())
}