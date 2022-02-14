use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::utils::{init_market, init_nft, mint_token};
use near_units::{parse_gas, parse_near};
use nft_bid_market::{ArgsKind, SaleArgs, SaleJson};

#[tokio::test]
async fn nft_on_approve_negative_tests() -> anyhow::Result<()> {
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

    // try to call nft_on_approve without cross contract call
    let outcome = user1
        .call(&worker, market.id().clone(), "nft_on_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "owner_id": user1.id(),
            "approval_id": 1,
            "msg": serde_json::json!(ArgsKind::Sale(SaleArgs {
                sale_conditions: HashMap::from([("near".parse().unwrap(), 10000.into())]),
                token_type: Some(series.clone()),
                start: None,
                end: None,
                origins: None,
            })).to_string()
        }))?
        .transact()
        .await?;
    match outcome.status {
        near_primitives::views::FinalExecutionStatus::Failure(err) => {
            assert!(err
                .to_string()
                .contains("nft_on_approve should only be called via cross-contract call"))
        }
        _ => panic!(),
    };

    // TODO: to test `owner_id` must be the signer need to create another contract

    // fail without storage deposit
    let outcome = user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Sale(SaleArgs {
                sale_conditions: HashMap::from([("near".parse().unwrap(), 10000.into())]),
                token_type: Some(series.clone()),
                start: None,
                end: None,
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    match outcome.status {
        near_primitives::views::FinalExecutionStatus::Failure(err) => {
            assert!(err.to_string().contains("Insufficient storage paid"))
        }
        _ => panic!(),
    };

    // not supported ft
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
            "msg": serde_json::json!(ArgsKind::Sale(SaleArgs {
                sale_conditions: HashMap::from([("ft.near".parse().unwrap(), 10000.into())]),
                token_type: Some(series),
                start: None,
                end: None,
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    match outcome.status {
        near_primitives::views::FinalExecutionStatus::Failure(err) => {
            assert!(err
                .to_string()
                .contains("Token ft.near not supported by this market"))
        }
        _ => panic!(),
    };

    Ok(())
}

#[tokio::test]
async fn nft_on_approve_positive() -> anyhow::Result<()> {
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
            "msg": serde_json::json!(ArgsKind::Sale(SaleArgs {
                sale_conditions: HashMap::from([("near".parse().unwrap(), 10000.into())]),
                token_type: Some(series.clone()),
                start: None,
                end: None,
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    let since_the_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let sale_json: SaleJson = market
        .view(
            &worker,
            "get_sale",
            serde_json::json!({ "nft_contract_token": format!("{}||{}", nft.id(), token1) })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    let time_passed = since_the_epoch - Duration::from_nanos(sale_json.start.unwrap().0);
    assert!(time_passed < Duration::from_secs(60)); // shouldn't be 60 secs even in worse case
    Ok(())
}
