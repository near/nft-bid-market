use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::utils::{init_market, init_nft, create_subaccount, create_series, deposit,
    mint_token, nft_approve, price_with_fees, offer,
    check_outcome_success, check_outcome_fail
};
use near_units::{parse_gas, parse_near};
use nft_bid_market::{ArgsKind, SaleArgs, SaleJson};
use nft_contract::common::{AccountId, U128};

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
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

    let user1 = create_subaccount(&worker, &owner, "user1").await?;
    let user2 = create_subaccount(&worker, &owner, "user2").await?;

    let series = create_series(&worker, nft.id().clone(), &user1, owner.id().clone()).await?;
    let token1 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;
    deposit(&worker, market.id().clone(), &user1).await;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 10000.into())]);
    nft_approve(&worker, nft.id().clone(), market.id().clone(), &user1, token1.clone(), sale_conditions.clone(), series.clone()).await;
    let price = price_with_fees(&worker, &market, sale_conditions.clone()).await?;
    offer(&worker, nft.id().clone(), market.id().clone(), &user2, token1.clone(), price.clone()).await;
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