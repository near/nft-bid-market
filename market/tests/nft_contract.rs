#![allow(clippy::ref_in_deref)]
mod utils;
use std::collections::HashMap;

use near_sdk_sim::{call, to_yocto, transaction::ExecutionStatus, view};
use nft_contract::{common::TokenMetadata, TokenSeriesJson};
use utils::init;

#[test]
fn nft_create_series_negative() {
    let (root, _, nft) = init();
    let user1 = root.create_user("user1".parse().unwrap(), to_yocto("1000"));
    let user2 = root.create_user("user2".parse().unwrap(), to_yocto("1000"));
    // Only authorized account can create series
    call!(root, nft.set_private_minting(true));
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
    let res = call!(
        user1,
        nft.nft_create_series(token_metadata.clone(), None),
        deposit = to_yocto("0.005")
    );
    if let ExecutionStatus::Failure(execution_error) =
        &res.promise_errors().remove(0).unwrap().outcome().status
    {
        assert!(execution_error
            .to_string()
            .contains("Access to mint is denied for this contract"));
    } else {
        panic!("Expected failure");
    }
    call!(root, nft.grant(user1.account_id()));

    // Title of the series should be specified
    let res = call!(
        user1,
        nft.nft_create_series(
            TokenMetadata {
                title: None,
                ..token_metadata.clone()
            },
            None
        ),
        deposit = to_yocto("0.005")
    );
    if let ExecutionStatus::Failure(execution_error) =
        &res.promise_errors().remove(0).unwrap().outcome().status
    {
        assert!(execution_error
            .to_string()
            .contains("title is missing from token metadata"));
    } else {
        panic!("Expected failure");
    }

    // Royalty can't exceed 50%
    let royalty = HashMap::from([(user1.account_id(), 500), (user2.account_id(), 5000)]);
    let res = call!(
        user1,
        nft.nft_create_series(token_metadata, Some(royalty)),
        deposit = to_yocto("0.005")
    );
    if let ExecutionStatus::Failure(execution_error) =
        &res.promise_errors().remove(0).unwrap().outcome().status
    {
        assert!(execution_error
            .to_string()
            .contains("maximum royalty cap exceeded"));
    } else {
        panic!("Expected failure");
    }
}

#[test]
fn nft_create_series_positive() {
    let (root, _, nft) = init();
    let user1 = root.create_user("user1".parse().unwrap(), to_yocto("1000"));
    let user2 = root.create_user("user2".parse().unwrap(), to_yocto("1000"));
    let royalty = HashMap::from([(user1.account_id(), 500), (user2.account_id(), 2000)]);

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
    call!(
        user1,
        nft.nft_create_series(token_metadata.clone(), None),
        deposit = to_yocto("0.005")
    ).assert_success();
    call!(root, nft.set_private_minting(true));
    // with private minting
    call!(root, nft.grant(user2.account_id()));
    let series_id: String = call!(
        user2,
        nft.nft_create_series(token_metadata.clone(), Some(royalty.clone())),
        deposit = to_yocto("1")
    ).unwrap_json();
    assert!(user2.account().unwrap().amount > to_yocto("999")); // make sure that deposit is refunded
    let series_json: TokenSeriesJson  = view!(nft.nft_get_series_json(series_id)).unwrap_json();
    //assert_eq!(series_json.royalty, royalty);
    assert_eq!(series_json, TokenSeriesJson {
        metadata: token_metadata,
        owner_id: user2.account_id(),
        royalty,
    })
}
