use std::collections::HashMap;

use crate::utils::init_nft;
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_units::parse_near;
use nft_contract::TokenSeriesJson;

#[tokio::test]
async fn nft_create_series_negative() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
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

    // Only authorized account can create series
    owner
        .call(&worker, nft.id().clone(), "set_private_minting")
        .args_json(serde_json::json!({
            "enabled": true,
        }))?
        .transact()
        .await?;
    let token_metadata = TokenMetadata {
        title: Some("some title".to_string()),
        description: None,
        media: Some("ipfs://QmTqZsmhZLLbi8vxZwm21wjKRFRBUQFzMFtTiyh3DJ2CCz".to_string()),
        media_hash: None,
        copies: Some(7),
        issued_at: None,
        expires_at: None,
        starts_at: None,
        updated_at: None,
        extra: None,
        reference: None,
        reference_hash: None,
    };
    let outcome = user1
        .call(&worker, nft.id().clone(), "nft_create_series")
        .args_json(serde_json::json!({
            "token_metadata": token_metadata,
            "royalty": null
        }))?
        .deposit(parse_near!("0.005 N"))
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err
            .to_string()
            .contains("Access to mint is denied for this contract"))
    } else {
        panic!("Expected failure")
    };
    owner
        .call(&worker, nft.id().clone(), "grant")
        .args_json(serde_json::json!({
            "account_id": user1.id()
        }))?
        .transact()
        .await?;

    // Title of the series should be specified
    let outcome = user1
        .call(&worker, nft.id().clone(), "nft_create_series")
        .args_json(serde_json::json!({
            "token_metadata": TokenMetadata{
                title: None,
                ..token_metadata.clone()},
            "royalty": null
        }))?
        .deposit(parse_near!("0.005 N"))
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err
            .to_string()
            .contains("title is missing from token metadata"))
    } else {
        panic!("Expected failure")
    };

    // Royalty can't exceed 50%
    let royalty = HashMap::from([(user1.id(), 500), (user2.id(), 5000)]);
    let outcome = user1
        .call(&worker, nft.id().clone(), "nft_create_series")
        .args_json(serde_json::json!({
            "token_metadata": token_metadata,
            "royalty": royalty,
        }))?
        .deposit(parse_near!("0.005 N"))
        .transact()
        .await?;
    if let near_primitives::views::FinalExecutionStatus::Failure(err) = outcome.status {
        assert!(err.to_string().contains("maximum royalty cap exceeded"))
    } else {
        panic!("Expected failure")
    };
    Ok(())
}

#[tokio::test]
async fn nft_create_series_positive() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let owner = worker.root_account();
    let nft = init_nft(&worker, owner.id()).await?;
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
    let royalty = HashMap::from([(user1.id(), 500), (user2.id(), 2000)]);
    let token_metadata = TokenMetadata {
        title: Some("some title".to_string()),
        description: None,
        media: Some("ipfs://QmTqZsmhZLLbi8vxZwm21wjKRFRBUQFzMFtTiyh3DJ2CCz".to_string()),
        media_hash: None,
        copies: Some(7),
        issued_at: None,
        expires_at: None,
        starts_at: None,
        updated_at: None,
        extra: None,
        reference: None,
        reference_hash: None,
    };
    let series1: String = user1
        .call(&worker, nft.id().clone(), "nft_create_series")
        .args_json(serde_json::json!({
            "token_metadata": token_metadata,
            "royalty": royalty,
        }))?
        .deposit(parse_near!("0.005 N"))
        .transact()
        .await?
        .json()?;

    owner
        .call(&worker, nft.id().clone(), "set_private_minting")
        .args_json(serde_json::json!({
            "enabled": true,
        }))?
        .transact()
        .await?;

    owner
        .call(&worker, nft.id().clone(), "grant")
        .args_json(serde_json::json!({
            "account_id": user2.id()
        }))?
        .transact()
        .await?;
    let series2: String = user2
        .call(&worker, nft.id().clone(), "nft_create_series")
        .args_json(serde_json::json!({
            "token_metadata": token_metadata,
            "royalty": royalty,
        }))?
        .deposit(parse_near!("0.005 N"))
        .transact()
        .await?
        .json()?;
    let series1_json: TokenSeriesJson = nft
        .view(
            &worker,
            "nft_get_series_json",
            serde_json::json!({ "token_series_id": series1 })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    let series2_json: TokenSeriesJson = nft
        .view(
            &worker,
            "nft_get_series_json",
            serde_json::json!({ "token_series_id": series2 })
                .to_string()
                .into_bytes(),
        )
        .await?
        .json()?;
    assert_eq!(series1_json.metadata, series2_json.metadata);
    // TODO: check balance of user1 after workspaces updated
    Ok(())
}
