use std::collections::HashMap;
use anyhow::Result;
use serde_json::json;

use crate::utils::{
    check_outcome_fail, check_outcome_success, create_series, create_subaccount, deposit,
    init_market, init_nft, mint_token, nft_approve, offer, offer_with_duration,
};
use near_units::parse_gas;
use nft_bid_market::{BidId, SaleJson};
use nft_contract::common::{U128, U64};

use crate::transaction_status::StatusCheck;
pub use workspaces::result::CallExecutionDetails;
/*
- TODO: Refunds a bid, removes it from the list
*/
#[tokio::test]
async fn remove_bid_positive() -> Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

    let user1 = create_subaccount(&worker, &owner, "user1").await?;
    let user2 = create_subaccount(&worker, &owner, "user2").await?;

    let series = create_series(&worker, nft.id(), &user1, owner.id()).await?;
    let token1 = mint_token(&worker, nft.id(), &user1, user1.id(), &series).await?;
    deposit(&worker, market.id(), &user1).await?;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 10000.into())]);
    nft_approve(
        &worker,
        nft.id(),
        market.id(),
        &user1,
        &token1,
        &sale_conditions,
        &series,
    )
    .await?;
    let price: U128 = 900.into();
    offer(
        &worker,
        nft.id(),
        market.id(),
        &user2,
        &token1,
        price,
    )
    .await?;

    // Check that one bid is removed after `remove_bid`
    let bids_by_owner: Vec<BidId> = market
        .view(
            &worker,
            "get_bids_id_by_account",
            json!({
                "owner_id": user2.id().to_string(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    /*assert!(
        bids_by_owner.len() == 1,
        "There should be exactly one bid"
    );

    let outcome = user2
        .call(&worker, market.id(), "remove_bid")
        .args_json(json!({
            "nft_contract_id": nft.id(),
            "token_id": &token1,
            "ft_token_id": "near",
            "price": price,
            "bid_id": 0,
        }))?
        .deposit(1)
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    /heck_outcome_success(outcome.status).await;

    let bids_by_owner: Vec<BidId> = market
        .view(
            &worker,
            "get_bids_id_by_account",
            json!({
                "owner_id": user2.id()
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert!(
        bids_by_owner.len() == 0,
        "Bid was not removed"
    );*/

    Ok(())
}

/*
- Should panic unless 1 yoctoNEAR is attached
- Should panic if there is no sale with the given `nft_contract_id` and `token_id`
- Should panic if there is no bids with `ft_token_id`
*/
#[tokio::test]
async fn remove_bid_negative() -> Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

    let user1 = create_subaccount(&worker, &owner, "user1").await?;
    let user2 = create_subaccount(&worker, &owner, "user2").await?;

    let series = create_series(&worker, nft.id(), &user1, owner.id()).await?;
    let token1 = mint_token(&worker, nft.id(), &user1, user1.id(), &series).await?;
    deposit(&worker, market.id(), &user1).await?;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 10000.into())]);
    nft_approve(
        &worker,
        nft.id(),
        market.id(),
        &user1,
        &token1,
        &sale_conditions,
        &series,
    )
    .await?;
    let price: U128 = 900.into();
    offer(
        &worker,
        nft.id(),
        market.id(),
        &user2,
        &token1,
        price,
    )
    .await?;

    // Should panic unless 1 yoctoNEAR is attached
    let outcome = user2
        .call(&worker, market.id(), "remove_bid")
        .args_json(json!({
            "nft_contract_id": nft.id(),
            "token_id": &token1,
            "ft_token_id": "near",
            "price": price,
            "bid_id": 0,
        }))?
        .deposit(2)
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    outcome
        .assert_err("Requires attached deposit of exactly 1 yoctoNEAR")
        .unwrap();

    // Should panic if there is no sale with the given `nft_contract_id` and `token_id`
    let outcome = user2
        .call(&worker, market.id(), "remove_bid")
        .args_json(json!({
            "nft_contract_id": "some_other_nft_contract".to_string(),
            "token_id": &token1,
            "ft_token_id": "near",
            "price": price,
            "bid_id": 0,
        }))?
        .deposit(1)
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    outcome
        .assert_err("No bid for this nft contract and ft token")
        .unwrap();

    let outcome = user2
        .call(&worker, market.id(), "remove_bid")
        .args_json(json!({
            "nft_contract_id": nft.id(),
            "token_id": "1:10",
            "ft_token_id": "near",
            "price": price,
            "bid_id": 0,
        }))?
        .deposit(1)
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    outcome
        .assert_err("No bid for this nft contract and ft token")
        .unwrap();

    // Should panic if there is no bids with `ft_token_id`
    let outcome = user2
        .call(&worker, market.id(), "remove_bid")
        .args_json(json!({
            "nft_contract_id": nft.id(),
            "token_id": &token1,
            "ft_token_id": "not_near",
            "price": price,
            "bid_id": 0,
        }))?
        .deposit(1)
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    outcome.assert_err("No token").unwrap();

    Ok(())
}

/*
TODO: Refunds a bid, removes it from the list
*/
#[tokio::test]
async fn cancel_bid_positive() -> Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

    let user1 = create_subaccount(&worker, &owner, "user1").await?;
    let user2 = create_subaccount(&worker, &owner, "user2").await?;
    let user3 = create_subaccount(&worker, &owner, "user3").await?;

    let series = create_series(&worker, nft.id(), &user1, owner.id()).await?;
    let token1 = mint_token(&worker, nft.id(), &user1, user1.id(), &series).await?;
    deposit(&worker, market.id(), &user1).await?;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 10000.into())]);
    nft_approve(
        &worker,
        nft.id(),
        market.id(),
        &user1,
        &token1,
        &sale_conditions,
        &series,
    )
    .await?;
    let price: U128 = 900.into();
    offer_with_duration(
        &worker,
        nft.id(),
        market.id(),
        &user2,
        &token1,
        price,
        U64(100000000),
    )
    .await?;

    // Check that one bid is removed after `cancel_bid`
    let bids_by_owner: Vec<BidId> = market
        .view(
            &worker,
            "get_bids_id_by_account",
            json!({
                "owner_id": user2.id()
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert!(bids_by_owner.len() == 1, "There should be exactly one bid");

    let outcome = user3
        .call(&worker, market.id(), "cancel_bid")
        .args_json(json!({
            "nft_contract_id": nft.id(),
            "token_id": &token1,
            "ft_token_id": "near",
            "owner_id": user2.id(),
            "price": price,
            "bid_id": 0,
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    //check_outcome_success(outcome.status).await;
    assert!(
        outcome.is_ok(),
        "Failed with error {}",
        outcome.err().unwrap()
    );

    let bids_by_owner: Vec<BidId> = market
        .view(
            &worker,
            "get_bids_id_by_account",
            json!({
                "owner_id": user2.id()
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert!(bids_by_owner.is_empty(), "Bid was not removed");

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
async fn cancel_bid_negative() -> Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

    let user1 = create_subaccount(&worker, &owner, "user1").await?;
    let user2 = create_subaccount(&worker, &owner, "user2").await?;
    let user3 = create_subaccount(&worker, &owner, "user3").await?;

    let series = create_series(&worker, nft.id(), &user1, owner.id()).await?;
    let token1 = mint_token(&worker, nft.id(), &user1, user1.id(), &series).await?;
    deposit(&worker, market.id(), &user1).await?;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 10000.into())]);
    nft_approve(
        &worker,
        nft.id(),
        market.id(),
        &user1,
        &token1,
        &sale_conditions,
        &series,
    )
    .await?;

    // Should panic if the bid isn't finished yet
    let price: U128 = 900.into();
    offer_with_duration(
        &worker,
        nft.id(),
        market.id(),
        &user2,
        &token1,
        price,
        U64(1000000000000),
    )
    .await?;

    let outcome = user3
        .call(&worker, market.id(), "cancel_bid")
        .args_json(json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
            "owner_id": user2.id(),
            "price": price,
            "bid_id": 0,
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    outcome.assert_err("The bid hasn't ended yet").unwrap();

    // Should panic if the bid doesn't have end time
    let price: U128 = 950.into();
    offer(
        &worker,
        nft.id(),
        market.id(),
        &user2,
        &token1,
        price,
    )
    .await?;

    let outcome = user3
        .call(&worker, market.id(), "cancel_bid")
        .args_json(json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
            "owner_id": user2.id(),
            "price": price,
            "bid_id": 1,
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    outcome.assert_err("The bid doesn't have an end").unwrap();

    // Should panic if the bid isn't finished yet
    let price: U128 = 900.into();
    offer_with_duration(
        &worker,
        nft.id(),
        market.id(),
        &user2,
        &token1,
        price,
        U64(1000000000000),
    )
    .await?;

    let outcome = user3
        .call(&worker, market.id(), "cancel_bid")
        .args_json(json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
            "owner_id": user2.id(),
            "price": price,
            "bid_id": 2,
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    outcome.assert_err("The bid hasn't ended yet").unwrap();

    // Should panic if there is no sale with the given `nft_contract_id` and `token_id`
    let price: U128 = 1000.into();
    offer_with_duration(
        &worker,
        nft.id(),
        market.id(),
        &user2,
        &token1,
        price,
        U64(100000000),
    )
    .await?;

    let outcome = user3
        .call(&worker, market.id(), "cancel_bid")
        .args_json(json!({
            "nft_contract_id": "another_nft_contract_id".to_string(),
            "token_id": token1,
            "ft_token_id": "near",
            "owner_id": user2.id(),
            "price": price,
            "bid_id": 2,
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    outcome.assert_err("No sale").unwrap();

    let outcome = user3
        .call(&worker, market.id(), "cancel_bid")
        .args_json(json!({
            "nft_contract_id": nft.id(),
            "token_id": "another_token_id".to_string(),
            "ft_token_id": "near",
            "owner_id": user2.id(),
            "price": price,
            "bid_id": 2,
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    outcome.assert_err("No sale").unwrap();

    // Should panic if there is no bids with `ft_token_id`
    let outcome = user3
        .call(&worker, market.id(), "cancel_bid")
        .args_json(json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "not_near",
            "owner_id": user2.id(),
            "price": price,
            "bid_id": 2,
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    outcome.assert_err("No token").unwrap();

    // Should panic if there is no bid with given `owner_id` and `price`
    let outcome = user3
        .call(&worker, market.id(), "cancel_bid")
        .args_json(json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
            "owner_id": user1.id(),
            "price": price,
            "bid_id": 2,
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    outcome.assert_err("No such bid").unwrap();

    let outcome = user3
        .call(&worker, market.id(), "cancel_bid")
        .args_json(json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
            "owner_id": user2.id(),
            "price": "1100",
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    outcome.assert_err("No such bid").unwrap();

    Ok(())
}

/*
- TODO: Refunds all expired bids, removes them from the list
*/
#[tokio::test]
async fn cancel_expired_bids_positive() -> Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

    let user1 = create_subaccount(&worker, &owner, "user1").await?;
    let user2 = create_subaccount(&worker, &owner, "user2").await?;
    let user3 = create_subaccount(&worker, &owner, "user3").await?;

    let series = create_series(&worker, nft.id(), &user1, owner.id()).await?;
    let token1 = mint_token(&worker, nft.id(), &user1, user1.id(), &series).await?;
    deposit(&worker, market.id(), &user1).await?;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 10000.into())]);
    nft_approve(
        &worker,
        nft.id(),
        market.id(),
        &user1,
        &token1,
        &sale_conditions,
        &series,
    )
    .await?;
    offer_with_duration(
        &worker,
        nft.id(),
        market.id(),
        &user2,
        &token1,
        U128(900),
        U64(100000000),
    )
    .await?;
    offer(
        &worker,
        nft.id(),
        market.id(),
        &user2,
        &token1,
        U128(950),
    )
    .await?;
    offer_with_duration(
        &worker,
        nft.id(),
        market.id(),
        &user2,
        &token1,
        U128(1000),
        U64(100000000),
    )
    .await?;

    // check that two bids are removed after `cancel_expired_bids`
    let bids_by_owner: Vec<BidId> = market
        .view(
            &worker,
            "get_bids_id_by_account",
            json!({
                "owner_id": user2.id()
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert!(
        bids_by_owner.len() == 3,
        "There should be exactly three bids"
    );

    let outcome = user3
        .call(&worker, market.id(), "cancel_expired_bids")
        .args_json(json!({
            "nft_contract_id": nft.id(),
            "token_id": &token1,
            "ft_token_id": "near",
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    //check_outcome_success(outcome.status).await;
    assert!(
        outcome.is_ok(),
        "Failed with error {}",
        outcome.err().unwrap()
    );

    let bids_by_owner: Vec<BidId> = market
        .view(
            &worker,
            "get_bids_id_by_account",
            json!({
                "owner_id": user2.id()
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    assert!(bids_by_owner.len() == 1, "There should be exactly two bids");

    Ok(())
}

/*
- Should panic if there is no sale with the given `nft_contract_id` and `token_id`
- Should panic if there is no bids with `ft_token_id`
*/
#[tokio::test]
async fn cancel_expired_bids_negative() -> Result<()> {
    let worker = workspaces::sandbox().await?;
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;

    let user1 = create_subaccount(&worker, &owner, "user1").await?;
    let user2 = create_subaccount(&worker, &owner, "user2").await?;
    let user3 = create_subaccount(&worker, &owner, "user3").await?;

    let series = create_series(&worker, nft.id(), &user1, owner.id()).await?;
    let token1 = mint_token(&worker, nft.id(), &user1, user1.id(), &series).await?;
    deposit(&worker, market.id(), &user1).await?;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 10000.into())]);
    nft_approve(
        &worker,
        nft.id(),
        market.id(),
        &user1,
        &token1,
        &sale_conditions,
        &series,
    )
    .await?;
    offer_with_duration(
        &worker,
        nft.id(),
        market.id(),
        &user2,
        &token1,
        U128(900),
        U64(100000000),
    )
    .await?;
    offer(
        &worker,
        nft.id(),
        market.id(),
        &user2,
        &token1,
        U128(950),
    )
    .await?;
    offer_with_duration(
        &worker,
        nft.id(),
        market.id(),
        &user2,
        &token1,
        U128(1000),
        U64(100000000),
    )
    .await?;

    // Should panic if there is no sale with the given `nft_contract_id` and `token_id`
    let outcome = user3
        .call(&worker, market.id(), "cancel_expired_bids")
        .args_json(json!({
            "nft_contract_id": "another_nft_contract".to_string(),
            "token_id": &token1,
            "ft_token_id": "near",
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    outcome.assert_err("No sale").unwrap();

    let outcome = user3
        .call(&worker, market.id(), "cancel_expired_bids")
        .args_json(json!({
            "nft_contract_id": nft.id(),
            "token_id": "another_token".to_string(),
            "ft_token_id": "near",
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    outcome.assert_err("No sale").unwrap();

    // Should panic if there is no bids with `ft_token_id`
    let outcome = user3
        .call(&worker, market.id(), "cancel_expired_bids")
        .args_json(json!({
            "nft_contract_id": nft.id(),
            "token_id": &token1,
            "ft_token_id": "not_near",
        }))?
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await;
    outcome.assert_err("No token").unwrap();

    Ok(())
}
