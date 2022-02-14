use near_units::parse_gas;
use serde_json::json;
use workspaces::prelude::*;
use workspaces::DevNetwork;

const NFT_WASM_FILEPATH: &str = "../res/nft_contract.wasm";
const MARKET_WASM_FILEPATH: &str = "../res/nft_bid_market.wasm";

pub const STORAGE_PRICE_PER_BYTE: u128 = 10_000_000_000_000_000_000;

pub async fn init_nft(
    worker: &workspaces::Worker<impl DevNetwork>,
    root_id: &workspaces::AccountId,
) -> anyhow::Result<workspaces::Contract> {
    let wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(wasm).await?;
    contract
        .call(worker, "new_default_meta")
        .args_json(serde_json::json!({
            "owner_id": root_id,
        }))?
        .gas(parse_gas!("150 Tgas") as u64)
        .transact()
        .await?;
    Ok(contract)
}

pub async fn init_market(
    worker: &workspaces::Worker<impl DevNetwork>,
    root_id: &workspaces::AccountId,
    nft_ids: Vec<workspaces::AccountId>,
) -> anyhow::Result<workspaces::Contract> {
    let wasm = std::fs::read(MARKET_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(wasm).await?;
    contract
        .call(worker, "new_default_meta")
        .args_json(serde_json::json!({
            "nft_ids": nft_ids,
            "owner_id": root_id,
        }))?
        .gas(parse_gas!("150 Tgas") as u64)
        .transact()
        .await?;
    Ok(contract)
}
