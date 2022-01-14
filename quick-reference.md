# NFT bid market

Change YOUR_ACCOUNT.testnet to your account:
```bash
export CONTRACT_PARENT=YOUR_ACCOUNT.testnet
```

`NFT_CONTRACT_ID` and `MARKET_CONTRACT_ID` will be used to deploy contracts:
```bash
export NFT_CONTRACT_ID=nft.$CONTRACT_PARENT
export MARKET_CONTRACT_ID=market.$CONTRACT_PARENT
```

If you are running this script at least for the second time and have already created these accounts, 
you should delete them:
```bash
set +e
near delete $NFT_CONTRACT_ID $CONTRACT_PARENT 2> /dev/null
near delete $MARKET_CONTRACT_ID $CONTRACT_PARENT 2> /dev/null
set -e
```
If you are running this script for the first time, the commands above should be omitted.

Create subaccounts `NFT_CONTRACT_ID` and `MARKET_CONTRACT_ID`:
```bash
near create-account $NFT_CONTRACT_ID --masterAccount $CONTRACT_PARENT --initialBalance 50
near create-account $MARKET_CONTRACT_ID --masterAccount $CONTRACT_PARENT --initialBalance 50
```

Deploy the contracts:
```bash
near deploy $NFT_CONTRACT_ID --wasmFile res/nft_contract.wasm
near deploy $MARKET_CONTRACT_ID --wasmFile res/nft_bid_market.wasm
```

Initialize contracts:
```bash
near call $NFT_CONTRACT_ID new_default_meta '{"owner_id": "'$CONTRACT_PARENT'", "market_id": "'$MARKET_CONTRACT_ID'"}' --accountId $NFT_CONTRACT_ID
near call $MARKET_CONTRACT_ID new '{"nft_ids": ["'$NFT_CONTRACT_ID'"], "owner_id": "'$CONTRACT_PARENT'"}' --accountId $MARKET_CONTRACT_ID
```
