# NFT bid market

NFT bid market consists of two contracts: NFT and Market.

NFT contract allows to create and manage a token or token series. 
It supports Metadata, Approval Management and Royalties.

Market contract handles sales, bids and auctions.

To build both contracts:
```bash
sh build-all.sh
```

To deploy it on dev account on testnet and export all the necessary variables:
```bash
sh deploy-testnet.sh
source .env
```

Now we have `CONTRACT_PARENT` and three subaccounts: `MARKET_CONTRACT_ID`, `NFT_CONTRACT_ID`, `ALICE`.
NFT contract is deployed on `NFT_CONTRACT_ID`.
Market contract is deployed on `CONTRACT_PARENT`.

Initialize contracts:
```bash
near call $NFT_CONTRACT_ID new_default_meta '{"owner_id": "'$CONTRACT_PARENT'", "market_id": "'$MARKET_CONTRACT_ID'"}' --accountId $NFT_CONTRACT_ID
near call $MARKET_CONTRACT_ID new '{"nft_ids": ["'$NFT_CONTRACT_ID'"], "owner_id": "'$CONTRACT_PARENT'"}' --accountId $MARKET_CONTRACT_ID
```

`CONTRACT_PARENT` creates the series of maximum five tokens:
```bash
near call $NFT_CONTRACT_ID nft_create_series '{"token_metadata": {"title": "some title", "media": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/00/Olympus_Mons_alt.jpg/1024px-Olympus_Mons_alt.jpg", "copies": 5}, "royalty": {"'$CONTRACT_PARENT'": 500}}' --accountId $CONTRACT_PARENT --deposit 0.005
```
And mints three of them:
```bash
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1
```

To create a sale the user needs to cover the storage:
```bash
near call $MARKET_CONTRACT_ID storage_deposit --accountId $CONTRACT_PARENT --deposit 0.02
```
In the future he can withdraw it:
```bash
near call $MARKET_CONTRACT_ID storage_withdraw --accountId $CONTRACT_PARENT --depositYocto 1
```

`CONTRACT_PARENT` puts one of the minted tokens on sale:
```bash
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:1", "account_id": "'$MARKET_CONTRACT_ID'", "msg": "{\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"is_auction\": false, \"start\": null, \"end\": null }"}' --accountId $CONTRACT_PARENT --deposit 1
```
Now any other account (in our case `ALICE`) can offer or buy the token:
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:1"}' --accountId $ALICE --depositYocto 10000 --gas 200000000000000
```
Alice attached the deposit equal to the price, so she did buy the token.

`CONTRACT_PARENT` puts the second of the minted tokens on sale:
```bash
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:2", "account_id": "'$MARKET_CONTRACT_ID'", "msg": "{\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"is_auction\": false, \"start\": null, \"end\": null }"}' --accountId $CONTRACT_PARENT --deposit 1
```

This time Alice attached less deposit:
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:2"}' --accountId $ALICE --depositYocto 8000 --gas 200000000000000
```

`ALICE` will get the token if `CONTRACT_PARENT` accepts the offer. To do it `CONTRACT_PARENT` should run this:
```bash
near call $MARKET_CONTRACT_ID accept_offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:2", "ft_token_id": "near"}' --accountId $CONTRACT_PARENT --gas 200000000000000
```
After this command `ALICE` receives the token.


`CONTRACT_PARENT` puts the trird of the minted tokens on sale:
```bash
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:3", "account_id": "'$MARKET_CONTRACT_ID'", "msg": "{\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"is_auction\": false, \"start\": null, \"end\": null }"}' --accountId $CONTRACT_PARENT --deposit 1
```

If `CONTRACT_PARENT` wants to increase (or decrease) the price, it can run 
```bash
near call $MARKET_CONTRACT_ID update_price '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:3", "ft_token_id": "near", "price": "12000"}' --accountId $CONTRACT_PARENT --depositYocto 1
```

Now the price is 12000 yoctoNear, so if `ALICE` tries to by it at a price 10000 yoctoNear she won't get it automatically add will need to wait for `CONTRACT_PARENT` to accept the offer.
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:3"}' --accountId $ALICE --depositYocto 10000 --gas 200000000000000
```

If `ALICE` decides to remove her bid: //doesn't work
```bash
near call $MARKET_CONTRACT_ID remove_bid '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:3", "bid": {"owner_id": "'$ALICE'", "price": "10000"}}' --accountId $ALICE --depositYocto 1
```

`CONTRACT_PARENT` can remove the sale:
```bash
near call $MARKET_CONTRACT_ID remove_sale '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:3"}' --accountId $CONTRACT_PARENT --depositYocto 1
```
