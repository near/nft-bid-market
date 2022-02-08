use std::collections::HashMap;

use near_sdk::AccountId;
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_sdk_sim::{
    call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount, STORAGE_AMOUNT,
};
//use nft_bid_market::common::*;
use nft_bid_market::MarketContract;
use nft_contract::NftContract;
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    NFT_WASM_BYTES => "../res/nft_bid_market.wasm",
    MARKET_WASM_BYTES => "../res/nft_contract.wasm",
}

const NFT_ID: &str = "nft";
const MARKET_ID: &str = "market";

pub fn init() -> (
    UserAccount,
    ContractAccount<MarketContract>,
    ContractAccount<NftContract>,
) {
    let root = init_simulator(None);

    let market = deploy!(
        contract: MarketContract,
        contract_id: MARKET_ID,
        bytes: &MARKET_WASM_BYTES,
        signer_account: root
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
        root,
        market.new(vec![NFT_ID.parse().unwrap()], root.account_id())
    ).assert_success();
    call!(
        user1,
        nft.nft_create_series(token_metadata, None)
        //deposit = to_yocto("0.005")
    ).assert_success();
}
