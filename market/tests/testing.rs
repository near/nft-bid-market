#![allow(clippy::ref_in_deref)]
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_sdk::{
    env,
    json_types::U128,
    serde_json::{self, json},
    AccountId,
};
use near_sdk_sim::{
    call, deploy, init_simulator,
    runtime::{init_runtime, GenesisConfig, RuntimeStandalone},
    to_yocto,
    transaction::ExecutionStatus,
    view, ContractAccount, UserAccount, STORAGE_AMOUNT,
};
use nft_bid_market::{AuctionJson, MarketContract, EXTENSION_DURATION};
use nft_contract::NftContract;
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    MARKET_WASM_BYTES => "../res/nft_bid_market.wasm",
    NFT_WASM_BYTES => "../res/nft_contract.wasm",
}

const NFT_ID: &str = "nft";
const MARKET_ID: &str = "market";

pub fn init() -> (
    UserAccount,
    ContractAccount<MarketContract>,
    ContractAccount<NftContract>,
) {
    let g_config = GenesisConfig {
        block_prod_time: 1_000_000_000 * 60, // 1 mins/block
        ..Default::default()
    };
    let root = init_simulator(Some(g_config));

    let market = deploy!(
        contract: MarketContract,
        contract_id: MARKET_ID,
        bytes: &MARKET_WASM_BYTES,
        signer_account: root,
        init_method: new(vec![NFT_ID.parse().unwrap()], root.account_id())
    );

    let nft = deploy!(
        contract: NftContract,
        contract_id: NFT_ID,
        bytes: &NFT_WASM_BYTES,
        signer_account: root,
        deposit: to_yocto("200"),
        init_method: new_default_meta(root.account_id(), market.account_id())
    );

    (root, market, nft)
}

fn prod_block(root: &UserAccount) {
    let mut runtime = root.borrow_runtime_mut();
    // println!("time: {}", runtime.current_block().block_timestamp);
    runtime.produce_block().unwrap();
    // println!("time: {}", runtime.current_block().block_timestamp);
}

#[test]
fn test_fees_transfers() {
    let (root, market, nft) = init();
    let origin1 = root.create_user("origin1".parse().unwrap(), to_yocto("1000"));
    let origin2 = root.create_user("origin2".parse().unwrap(), to_yocto("1000"));
    let origin3 = root.create_user("origin3".parse().unwrap(), to_yocto("1000"));
    let user1 = root.create_user("user1".parse().unwrap(), to_yocto("1000"));
    let user2 = root.create_user("user2".parse().unwrap(), to_yocto("1000"));
    let user3 = root.create_user("user3".parse().unwrap(), to_yocto("1000"));

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
    let royalty = HashMap::from([(user1.account_id(), 500)]);
    call!(
        user1,
        nft.nft_create_series(token_metadata, Some(royalty)),
        deposit = to_yocto("0.005")
    )
    .assert_success();

    for _ in 0..5 {
        call!(
            user1,
            nft.nft_mint("1".to_string(), user1.account_id(), None),
            deposit = to_yocto("0.01")
        )
        .assert_success();
    }

    call!(
        user1,
        market.storage_deposit(None),
        deposit = to_yocto("0.1")
    )
    .assert_success();
    let origins = HashMap::from([(origin1.account_id(), 100u32)]);
    for i in 1..3 {
        let token_id = format!("1:{}", i);
        call!(
            user1,
            nft.nft_approve(
                token_id,
                market.account_id(),
                Some(
                    json!({
                        "Auction": {
                            "token_type": "near",
                            "minimal_step": "100",
                            "start_price": "10000",
                            "start": null,
                            "duration": "900000000000",
                            "buy_out_price": "10000000000",
                            "origins": origins,
                        }
                    })
                    .to_string()
                )
            ),
            deposit = to_yocto("1")
        )
        .assert_success();
    }

    call!(user1, market.cancel_auction(0.into()), deposit = 1).assert_success();
    let time_during_bid = root.borrow_runtime().current_block().block_timestamp + root.borrow_runtime().genesis.block_prod_time; // +1 block
    call!(
        user2,
        market.auction_add_bid(1.into(), Some("near".to_string()), None),
        deposit = 10400
    )
    .assert_success();
    let res = call!(user1, market.finish_auction(1.into()));
    if let ExecutionStatus::Failure(execution_error) =
        &res.promise_errors().remove(0).unwrap().outcome().status
    {
        assert!(execution_error
            .to_string()
            .contains("Auction can be finalized only after the end time"));
    } else {
        unreachable!();
    }
    let auction_json: AuctionJson = view!(market.get_auction_json(1.into())).unwrap_json();
    assert!(auction_json.end.0 - time_during_bid == EXTENSION_DURATION);
    let blocks_needed = (auction_json.end.0 - root.borrow_runtime().current_block().block_timestamp) / root.borrow_runtime().genesis.block_prod_time;
    let mut i = 0;
    while root.borrow_runtime().current_block().block_timestamp < auction_json.end.0 {
        i += 1;
        prod_block(&root);
    }
    call!(user1, market.finish_auction(1.into())).assert_success();
    assert!(i == blocks_needed);
    println!("{}", serde_json::to_string_pretty(&auction_json).unwrap());
}
