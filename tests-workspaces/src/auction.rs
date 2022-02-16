use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::utils::{init_market, init_nft, mint_token, check_outcome_success, check_outcome_fail};
use near_units::{parse_gas, parse_near};
use nft_bid_market::{ArgsKind, AuctionJson};
use nft_bid_market::AuctionArgs;
use workspaces::{Contract, Account, Worker};

#[tokio::test]
async fn nft_on_approve_auction_positive() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

    let user1 = owner
        .create_subaccount(&worker, "user1")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();

    let series: String = user1
        .call(&worker, nft.id().clone(), "nft_create_series")
        .args_json(serde_json::json!({
        "token_metadata":
        {
            "title": "some title",
            "media": "ipfs://QmTqZsmhZLLbi8vxZwm21wjKRFRBUQFzMFtTiyh3DJ2CCz",
            "copies": 10
        },
        "royalty":
        {
            owner.id().as_ref(): 1000
        }}))?
        .deposit(parse_near!("0.005 N"))
        .transact()
        .await?
        .json()?;
    let token1 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;

    user1
        .call(&worker, market.id().clone(), "storage_deposit")
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    let outcome = user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Auction(AuctionArgs {
                token_type: None,
                minimal_step: 100.into(),
                start_price: 10000.into(),
                start: None,
                duration: 900000000000.into(),
                buy_out_price: Some(10000000000.into()),
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    check_outcome_success(outcome.status).await;
    Ok(())
}


#[tokio::test]
async fn auction_add_bid_negative() -> anyhow::Result<()> {
    /*
    - Should panic if `ft_token_id` is not supported
    - Should panic if the auction is not in progress
    - Should panic if the bid is smaller than the minimal deposit
    - Should panic if the bid is smaller than the previous one
    - Refunds a previous bid (if it exists)
    - Extends an auction if the bid is added less than 15 minutes before the end
    - The auction ends if the `attached_deposit` is bigger than the `buy_out_price` (plus fees)

    - can bid on its own auction?
    */
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

    let user1 = owner
        .create_subaccount(&worker, "user1")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    let user2 = owner
        .create_subaccount(&worker, "user2")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();

    let series: String = user1
        .call(&worker, nft.id().clone(), "nft_create_series")
        .args_json(serde_json::json!({
        "token_metadata":
        {
            "title": "some title",
            "media": "ipfs://QmTqZsmhZLLbi8vxZwm21wjKRFRBUQFzMFtTiyh3DJ2CCz",
            "copies": 10
        },
        "royalty":
        {
            owner.id().as_ref(): 1000
        }}))?
        .deposit(parse_near!("0.005 N"))
        .transact()
        .await?
        .json()?;
    let token1 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;

    user1
        .call(&worker, market.id().clone(), "storage_deposit")
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Auction(AuctionArgs {
                token_type: None,
                minimal_step: 100.into(),
                start_price: 10000.into(),
                start: None,
                duration: 900000000000.into(),
                buy_out_price: Some(10000000000.into()),
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;

    // Should panic if `ft_token_id` is not supported
    let outcome = user2
        .call(&worker, market.id().clone(), "auction_add_bid")
        .args_json(serde_json::json!({
            "auction_id": "0".to_string(),
            "token_type": "not_near".to_string(),
        }))?
        .deposit(10300)
        //.gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "token not supported").await;
    Ok(())
}