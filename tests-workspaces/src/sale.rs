use crate::utils::{init_market, init_nft};

#[tokio::test]
async fn offers_negative_tests() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();
    let nft = init_nft(&worker, worker.root_account().id()).await?;

    let market = init_market(&worker, worker.root_account().id(), vec![nft.id()]).await?;
    
    Ok(())
}