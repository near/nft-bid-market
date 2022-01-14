# NFT bid market

Change YOUR_ACCOUNT to your account(alice.testnet for example):
```bash
CONTRACT_PARENT=YOUR_ACCOUNT
```

`NFT_CONTRACT_ID` and `MARKET_CONTRACT_ID` will be used to deploy contracts:
```bash
NFT_CONTRACT_ID=nft.$CONTRACT_PARENT
MARKET_CONTRACT_ID=market.$CONTRACT_PARENT
ALICE=alice.$CONTRACT_PARENT
```

If you are running this script at least for the second time and have already created these accounts, 
you should delete them:
```bash
near delete $NFT_CONTRACT_ID $CONTRACT_PARENT 2> /dev/null
near delete $MARKET_CONTRACT_ID $CONTRACT_PARENT 2> /dev/null
near delete $ALICE $CONTRACT_PARENT 2> /dev/null
```
If you are running this script for the first time, the commands above should be omitted.

Create subaccounts `NFT_CONTRACT_ID` and `MARKET_CONTRACT_ID`:
```bash
near create-account $NFT_CONTRACT_ID --masterAccount $CONTRACT_PARENT --initialBalance 50
near create-account $MARKET_CONTRACT_ID --masterAccount $CONTRACT_PARENT --initialBalance 50
near create-account $ALICE --masterAccount $CONTRACT_PARENT --initialBalance 20
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

`CONTRACT_PARENT` creates the series of maximum three tokens:
```bash
near call $NFT_CONTRACT_ID nft_create_series '{"token_metadata": {"title": "some title", "media": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/00/Olympus_Mons_alt.jpg/1024px-Olympus_Mons_alt.jpg", "copies": 3}, "royalty": {"'$CONTRACT_PARENT'": 500}}' --accountId $CONTRACT_PARENT --deposit 0.005
```
And we mint two of them:
```bash
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1
```

To create a sale the user needs to cover the storage:
```bash
near call $MARKET_CONTRACT_ID storage_deposit --accountId $CONTRACT_PARENT --deposit 0.02
```

`CONTRACT_PARENT` puts one of the minted tokens of sale:
```bash
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:1", "account_id": "'$MARKET_CONTRACT_ID'", "msg": "{\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"is_auction\": false, \"start\": null, \"end\": null }"}' --accountId $CONTRACT_PARENT --deposit 1
```
Now any other account (in our case `ALICE`) can offer or buy the token:
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:1"}' --accountId $ALICE --depositYocto 10000 --gas 200000000000000
```
Alice attached exact amount, so she did buy token

`CONTRACT_PARENT` puts second one of the minted tokens of sale:
```bash
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:2", "account_id": "'$MARKET_CONTRACT_ID'", "msg": "{\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"is_auction\": false, \"start\": null, \"end\": null }"}' --accountId $CONTRACT_PARENT --deposit 1
```

This time alice offers less, then exact amount:
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:2"}' --accountId $ALICE --depositYocto 8000 --gas 200000000000000
```

`ALICE` will get the token if `CONTRACT_PARENT` accepts the offer. To do so is runs:
```bash
near call $MARKET_CONTRACT_ID accept_offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:2", "ft_token_id": "near"}' --accountId $CONTRACT_PARENT --gas 200000000000000
```
After that command `ALICE` receives a token.
