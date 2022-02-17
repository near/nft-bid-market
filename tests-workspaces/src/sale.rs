use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::utils::{init_market, init_nft, mint_token};
use near_contract_standards::non_fungible_token::Token;
use near_units::{parse_gas, parse_near};
use nft_bid_market::{ArgsKind, SaleArgs, SaleJson, BID_HISTORY_LENGTH_DEFAULT};
use nft_contract::common::{AccountId, U128, U64};

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
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err
            .to_string()
            .contains("nft_on_approve should only be called via cross-contract call"))
    } else {
        panic!("Expected failure")
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
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err.to_string().contains("Insufficient storage paid"))
    } else {
        panic!("Expected failure")
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
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err
            .to_string()
            .contains("Token ft.near not supported by this market"))
    } else {
        panic!("Expected failure")
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
    user1
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

/**
    - Should panic if there is no sale with given `contract_and_token_id`
    - Should panic if the sale is not in progress
    - Should panic if the NFT owner tries to make a bid on his own sale
    - Should panic if the deposit equal to 0
    - Should panic if the NFT can't be bought by `ft_token_id`
- If the `attached_deposit` is equal to the price + fees
  -  panics if number of payouts plus number of bids exceeds 10
- If the `attached_deposit` is not equal to the price + fees
  - should panic if `ft_token_id` is not supported
  - panics if the bid smaller or equal to the previous one
  - panic if origin fee exceeds ORIGIN_FEE_MAX
    */
#[tokio::test]
async fn offer_negative() -> anyhow::Result<()> {
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

    // No sale with given `contract_and_token_id`
    let outcome = user1
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": "1:1",
            "ft_token_id": "near",
        }))?
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err.to_string().contains("No sale"))
    } else {
        panic!("Expected failure")
    };

    // Sale is not in progress
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
    let since_the_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let waiting_time = Duration::from_secs(15);
    let epoch_plus_waiting_time = (since_the_epoch + waiting_time).as_nanos();
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 10000.into())]);
    user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Sale(SaleArgs {
                sale_conditions: sale_conditions.clone(),
                token_type: Some(series.clone()),
                start: Some(U64(epoch_plus_waiting_time as u64)),
                end: None,
                origins: None,
            })).to_string()
        }))?
        .deposit(parse_near!("1 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
    let outcome = user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err
            .to_string()
            .contains("Either the sale is finished or it hasn't started yet"))
    } else {
        panic!("Expected failure")
    };

    tokio::time::sleep(waiting_time).await;
    let price: U128 = market
        .view(
            &worker,
            "price_with_fees",
            serde_json::json!({
                "price": sale_conditions.get(&AccountId::new_unchecked("near".to_string())).unwrap(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    // NFT owner tries to make a bid on his own sale
    let outcome = user1
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .deposit(price.into())
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err.to_string().contains("Cannot bid on your own sale."))
    } else {
        panic!("Expected failure")
    };

    // Deposit equal to 0
    let outcome = user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .deposit(0)
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err
            .to_string()
            .contains("Attached deposit must be greater than 0"))
    } else {
        panic!("Expected failure")
    };

    // Not supported ft
    let outcome = user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "nearcoin",
        }))?
        .deposit(1000)
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err.to_string().contains("Not supported ft"))
    } else {
        panic!("Expected failure")
    };

    // the bid smaller or equal to the previous one
    user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .deposit(500)
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    let outcome = user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .deposit(400) // less
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err
            .to_string()
            .contains("Can't pay less than or equal to current bid price:"))
    } else {
        panic!("Expected failure")
    };
    let outcome = user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .deposit(500) // equal
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err
            .to_string()
            .contains("Can't pay less than or equal to current bid price:"))
    } else {
        panic!("Expected failure")
    };

    // Exceeding ORIGIN_FEE_MAX
    let outcome = user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
            "origins": {
                "user1": 4701,
            }
        }))?
        .deposit(2000) // equal
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err.to_string().contains("Max origins exceeded"))
    } else {
        panic!("Expected failure")
    };

    // number of payouts plus number of bids exceeds 10
    let too_much_origins: HashMap<AccountId, u32> = HashMap::from([
        ("acc1.near".parse().unwrap(), 100),
        ("acc2.near".parse().unwrap(), 100),
        ("acc3.near".parse().unwrap(), 100),
        ("acc4.near".parse().unwrap(), 100),
        ("acc5.near".parse().unwrap(), 100),
        ("acc6.near".parse().unwrap(), 100),
        ("acc7.near".parse().unwrap(), 100),
        ("acc8.near".parse().unwrap(), 100),
        ("acc9.near".parse().unwrap(), 100),
        ("acc10.near".parse().unwrap(), 100),
        ("acc11.near".parse().unwrap(), 100),
        ("acc12.near".parse().unwrap(), 100),
    ]);
    let price: U128 = market
        .view(
            &worker,
            "price_with_fees",
            serde_json::json!({
                "price": sale_conditions.get(&AccountId::new_unchecked("near".to_string())).unwrap(),
                "origins": too_much_origins
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    let outcome = user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .deposit(price.into())
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    // Promise of offer returning empty value, because of panic on nft_transfer_payout, but
    // TODO: we need to check Failure on nft contract when workspaces add feature to check not only FinalExecutionStatus
    if let near_primitives::views::FinalExecutionStatus::SuccessValue(empty_string) = outcome.status
    {
        assert!(empty_string.is_empty())
    } else {
        panic!("Expected failure {:?}", outcome.status)
    };

    Ok(())
}

/*
- If the `attached_deposit` is equal to the price + fees
    -  NFT is transferred to the buyer
    -  the sale is removed from the list of sales
    -  ft transferred to the previous owner
    -  protocol, royalty and origin fees are paid
    -  royalty paid from seller side
    -  previous bids refunded
- If the `attached_deposit` is not equal to the price + fees
  - a new bid should be added
  - if the number of stored bids exceeds `bid_history_length`, the earliest bid is removed and refunded
*/
#[tokio::test]
async fn offer_positive() -> anyhow::Result<()> {
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

    let sale_conditions = HashMap::from([("near".parse().unwrap(), 10000.into())]);
    user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token1,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Sale(SaleArgs {
                sale_conditions: sale_conditions.clone(),
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
    let price: U128 = market
        .view(
            &worker,
            "price_with_fees",
            serde_json::json!({
                "price": sale_conditions.get(&AccountId::new_unchecked("near".to_string())).unwrap(),
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    let nft_contract_token = format!("{}||{}", nft.id(), token1);

    let before_sell: Option<SaleJson> = market
        .view(
            &worker,
            "get_sale",
            serde_json::json!({
                "nft_contract_token": nft_contract_token,
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token1,
            "ft_token_id": "near",
        }))?
        .deposit(price.into())
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;

    let token: Token = nft
        .view(
            &worker,
            "nft_token",
            serde_json::json!({ "token_id": token1 })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;

    // NFT is transferred to the buyer
    assert_eq!(token.owner_id.as_str(), user2.id().as_ref());
    // the sale is removed from the list of sales
    let after_sell: Option<SaleJson> = market
        .view(
            &worker,
            "get_sale",
            serde_json::json!({
                "nft_contract_token": nft_contract_token,
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;

    assert!(
        after_sell.is_none(),
        "Sale is still active, when it shouldn't"
    );
    assert!(
        before_sell.is_some(),
        "Sale is not active, when it should be"
    );

    // Check if bids can be added
    let token2 = mint_token(&worker, nft.id().clone(), &user1, user1.id(), &series).await?;
    let sale_conditions = HashMap::from([("near".parse().unwrap(), 10000.into())]);
    user1
        .call(&worker, nft.id().clone(), "nft_approve")
        .args_json(serde_json::json!({
            "token_id": token2,
            "account_id": market.id(),
            "msg": serde_json::json!(ArgsKind::Sale(SaleArgs {
                sale_conditions: sale_conditions.clone(),
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
    let initial_price = 100;
    user2
        .call(&worker, market.id().clone(), "offer")
        .args_json(serde_json::json!({
            "nft_contract_id": nft.id(),
            "token_id": token2,
            "ft_token_id": "near",
        }))?
        .deposit(initial_price)
        .gas(parse_gas!("300 Tgas") as u64)
        .transact()
        .await?;
    let nft_contract_token = format!("{}||{}", nft.id(), token2);
    let sale_json: SaleJson = market
        .view(
            &worker,
            "get_sale",
            serde_json::json!({
                "nft_contract_token": nft_contract_token,
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    println!("net");
    let bids = sale_json
        .bids
        .get(&AccountId::new_unchecked("near".to_string()))
        .unwrap();
    assert!(bids.get(0).is_some(), "Bid not added");

    let first_bid = bids.get(0).unwrap();
    // Earliest bid should be removed
    for i in 1..=BID_HISTORY_LENGTH_DEFAULT {
        user2
            .call(&worker, market.id().clone(), "offer")
            .args_json(serde_json::json!({
                "nft_contract_id": nft.id(),
                "token_id": token2,
                "ft_token_id": "near",
            }))?
            .deposit(initial_price * (i + 1) as u128)
            .gas(parse_gas!("300 Tgas") as u64)
            .transact()
            .await?;
        let sale_json: SaleJson = market
            .view(
                &worker,
                "get_sale",
                serde_json::json!({
                    "nft_contract_token": nft_contract_token,
                })
                .to_string()
                .into_bytes(),
            )
            .await?
            .json()?;
        let bids = sale_json
            .bids
            .get(&AccountId::new_unchecked("near".to_string()))
            .unwrap();
        if i < BID_HISTORY_LENGTH_DEFAULT {
            assert_eq!(bids.get(0).unwrap(), first_bid);
        }
    }
    // new bid removed last bid
    let sale_json: SaleJson = market
        .view(
            &worker,
            "get_sale",
            serde_json::json!({
                "nft_contract_token": nft_contract_token,
            })
            .to_string()
            .into_bytes(),
        )
        .await?
        .json()?;
    let bids = sale_json
        .bids
        .get(&AccountId::new_unchecked("near".to_string()))
        .unwrap();
    assert_ne!(bids.get(0).unwrap(), first_bid);
    Ok(())
}
