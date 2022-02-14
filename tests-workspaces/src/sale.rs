use std::collections::HashMap;

use crate::utils::{init_market, init_nft, mint_token};
use near_units::parse_near;
use nft_bid_market::{ArgsKind, SaleArgs};

#[tokio::test]
async fn offers_negative_tests() -> anyhow::Result<()> {
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
            owner.id().as_ref(): 500
        }}))?
        .deposit(parse_near!("0.005 N"))
        .transact()
        .await?
        .json()?;
    let token1 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;
    // fail without storage deposit
    let sale = ArgsKind::Sale(SaleArgs {
        sale_conditions: HashMap::from([("near".parse().unwrap(), 10000.into())]),
        token_type: Some("1".to_owned()),
        start: None,
        end: None,
        origins: None,
    });
    let outcome = user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": "1:1",
            "account_id": "'$MARKET_CONTRACT_ID'",
            "msg": serde_json::json!(sale)
        }))?
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    match outcome.status {
        near_primitives::views::FinalExecutionStatus::Failure(err) => println!("err: {}", err),
        _ => panic!(),
    };
    user1
        .call(&worker, market.id().clone(), "storage_deposit")
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    Ok(())
}
