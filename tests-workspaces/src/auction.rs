use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::utils::{init_market, init_nft, mint_token, check_outcome_success, check_outcome_fail};
use near_units::{parse_gas, parse_near};
use nft_bid_market::{ArgsKind, AuctionArgs, AuctionJson};
//use workspaces::{Contract, Account, Worker};

const THIRTY_SECONDS: Duration = Duration::from_secs(30);
const FIFTEEN_MINUTES: Duration = Duration::from_secs(60 * 15);

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
    - TODO: Should panic if the auction is not in progress
    - Should panic if the bid is smaller than the minimal deposit
    - Should panic if the bid is smaller than the previous one + minimal step + fees

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
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "token not supported").await;

    // Should panic if the bid is smaller than the minimal deposit
    let outcome = user2
        .call(&worker, market.id().clone(), "auction_add_bid")
        .args_json(serde_json::json!({
            "auction_id": "0".to_string(),
        }))?
        .deposit(10200)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "Should bid at least 10300").await;

    // Should panic if the bid is smaller than the previous one
    user2
        .call(&worker, market.id().clone(), "auction_add_bid")
        .args_json(serde_json::json!({
            "auction_id": "0".to_string(),
        }))?
        .deposit(10300)
        .transact()
        .await?;
    let outcome = user2
        .call(&worker, market.id().clone(), "auction_add_bid")
        .args_json(serde_json::json!({
            "auction_id": "0".to_string(),
        }))?
        .deposit(10350)
        .transact()
        .await?;
    //println!("outcome: {:?}", outcome);
    check_outcome_fail(outcome.status, "Should bid at least 10403").await;

    Ok(())
}

#[tokio::test]
async fn auction_add_bid_positive() -> anyhow::Result<()> {
    /*
    - TODO: Refunds a previous bid (if it exists)
    - Extends an auction if the bid is added less than 15 minutes before the end
    - The auction ends if the `attached_deposit` is bigger than the `buy_out_price` (plus fees)
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

    // Extends an auction if the bid is added less than 15 minutes before the end
    let auction: AuctionJson = market
        .view(
            &worker,
            "get_auction_json",
            serde_json::json!({ "auction_id": "0".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    let right_before_bid = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    user2
        .call(&worker, market.id().clone(), "auction_add_bid")
        .args_json(serde_json::json!({
            "auction_id": "0".to_string(),
        }))?
        .deposit(10300)
        .transact()
        .await?;
    let auction_bought_out: AuctionJson = market
        .view(
            &worker,
            "get_auction_json",
            serde_json::json!({ "auction_id": "0".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert!(auction.end.0 < auction_bought_out.end.0, "The auction wasn't extended");
    assert!(Duration::from_nanos(auction_bought_out.end.0) - (right_before_bid  + FIFTEEN_MINUTES) < THIRTY_SECONDS);

    // The auction ends if the `attached_deposit` is bigger than the `buy_out_price` (plus fees)
    let auction: AuctionJson = market
        .view(
            &worker,
            "get_auction_json",
            serde_json::json!({ "auction_id": "0".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    let right_before_bid = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    user2
        .call(&worker, market.id().clone(), "auction_add_bid")
        .args_json(serde_json::json!({
            "auction_id": "0".to_string(),
        }))?
        .deposit(10300000000)
        .transact()
        .await?;
    let auction_bought_out: AuctionJson = market
        .view(
            &worker,
            "get_auction_json",
            serde_json::json!({ "auction_id": "0".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    let in_progress: bool = market
        .view(
            &worker,
            "check_auction_in_progress",
            serde_json::json!({ "auction_id": "0".to_string() })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert!(auction.end.0 > auction_bought_out.end.0, "The end time wasn't decreased");
    assert!(!in_progress, "The auction didn't end");
    assert!(Duration::from_nanos(auction_bought_out.end.0) - right_before_bid < THIRTY_SECONDS);
    Ok(())
}

#[tokio::test]
async fn cancel_auction_negative() -> anyhow::Result<()> {
    /*
    - Should panic unless 1 yoctoNEAR is attached
    - Can only be called by the creator of the auction
    - TODO: Panics if auction is not active
    - Panics if the auction already has a bid
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
    
    // Should panic unless 1 yoctoNEAR is attached
    let outcome = user1
        .call(&worker, market.id().clone(), "cancel_auction")
        .args_json(serde_json::json!({
            "auction_id": "0".to_string()
        }))?
        .deposit(2)
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "Requires attached deposit of exactly 1 yoctoNEAR").await;

    // Can only be called by the creator of the auction
    let outcome = user2
        .call(&worker, market.id().clone(), "cancel_auction")
        .args_json(serde_json::json!({
            "auction_id": "0".to_string()
        }))?
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "Only the auction owner can cancel the auction").await;

    // Panics if the auction already has a bid
    user2
        .call(&worker, market.id().clone(), "auction_add_bid")
        .args_json(serde_json::json!({
            "auction_id": "0".to_string(),
        }))?
        .deposit(10300)
        .transact()
        .await?;
    let outcome = user1
        .call(&worker, market.id().clone(), "cancel_auction")
        .args_json(serde_json::json!({
            "auction_id": "0".to_string()
        }))?
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    check_outcome_fail(outcome.status, "Can't cancel the auction after the first bid is made").await;

    let vector_auctions: Vec<AuctionJson> = market.view(
        &worker,
        "get_auctions",
        serde_json::json!({"from_index": null, "limit": null})
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert!(!vector_auctions.is_empty(), "Deleted the auction");
    Ok(())
}

#[tokio::test]
async fn cancel_auction_positive() -> anyhow::Result<()> {
    /*
    - Removes the auction
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

    let outcome = user1
        .call(&worker, market.id().clone(), "cancel_auction")
        .args_json(serde_json::json!({
            "auction_id": "0".to_string()
        }))?
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    check_outcome_success(outcome.status).await;
    let vector_auctions: Vec<AuctionJson> = market.view(
        &worker,
        "get_auctions",
        serde_json::json!({})
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert!(vector_auctions.is_empty(), "Did not delete the auction");
    Ok(())
}

