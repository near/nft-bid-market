build
```bash
sh deploy-testnet.sh
source .env
```
init
```bash
near call $NFT_CONTRACT_ID new_default_meta '{"owner_id": "'$CONTRACT_PARENT'", "market_id": "'$MARKET_CONTRACT_ID'"}' --accountId $NFT_CONTRACT_ID
near call $MARKET_CONTRACT_ID new '{"nft_ids": ["'$NFT_CONTRACT_ID'"], "owner_id": "'$CONTRACT_PARENT'"}' --accountId $MARKET_CONTRACT_ID
```
create nft
```bash
near call $NFT_CONTRACT_ID nft_create_series '{"token_metadata": {"title": "some title", "media": "ipfs://QmTqZsmhZLLbi8vxZwm21wjKRFRBUQFzMFtTiyh3DJ2CCz", "copies": 7}, "royalty": 
{"'$CONTRACT_PARENT'": 500}}' --accountId $CONTRACT_PARENT --deposit 0.005

near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 0.01
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 0.01
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 0.01
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 0.01
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 0.01
```
check what we minted
```bash
near view $NFT_CONTRACT_ID nft_token '{"token_id": "1:1"}'
```

# SALE
put it on sale
```bash
near call $MARKET_CONTRACT_ID storage_deposit --accountId $CONTRACT_PARENT --deposit 0.1
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:1", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Sale\": {\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"start\": null, \"end\": null, \"origins\": {\"'$NFT_CONTRACT_ID'\": 100}} }"}' --accountId $CONTRACT_PARENT --deposit 0.00044
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:2", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Sale\": {\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"start\": null, \"end\": null, \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 0.00044

near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:3", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Sale\": {\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"start\": null, \"end\": null, \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 0.00044
```

lets check sale on nft contract
```bash
near view $MARKET_CONTRACT_ID get_sale '{"nft_contract_token": "'$NFT_CONTRACT_ID'||1:1"}'
```

alice buys it for this price
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:1", "ft_token_id": "near"}' --accountId $ALICE --depositYocto 10300 --gas 200000000000000
```

let's place a bid
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:2", "ft_token_id": "near"}' --accountId $ALICE --depositYocto 10200 --gas 200000000000000
```

and accept offer
```bash
near call $MARKET_CONTRACT_ID accept_offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:2", "ft_token_id": "near"}' --accountId $CONTRACT_PARENT --gas 200000000000000
```

update price on another sale
```bash
near call $MARKET_CONTRACT_ID update_price '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:3", "ft_token_id": "near", "price": "12000"}' --accountId $CONTRACT_PARENT --depositYocto 1

near view $MARKET_CONTRACT_ID get_sale '{"nft_contract_token": "'$NFT_CONTRACT_ID'||1:3"}'
```

we can remove bids
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:3", "ft_token_id": "near"}' --accountId $ALICE --depositYocto 10000 --gas 200000000000000

near call $MARKET_CONTRACT_ID remove_bid '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:3", "ft_token_id": "near", "bid": {"owner_id": "'$ALICE'", "price": "10000", "origins": {}}}' --accountId $ALICE --depositYocto 1
```

sale can be removed and all bids will get returned
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:3", "ft_token_id": "near"}' --accountId $ALICE --depositYocto 10000 --gas 200000000000000

near call $MARKET_CONTRACT_ID remove_sale '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:3"}' --accountId $CONTRACT_PARENT --depositYocto 1
```

# AUCTION

now lets put tokens on auction
```bash
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:4", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Auction\": {\"token_type\": \"near\", \"minimal_step\": \"100\", \"start_price\": \"10000\", \"start\": null, \"duration\": \"60000000000\", \"buy_out_price\": \"10000000000\", \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:5", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Auction\": {\"token_type\": \"near\", \"minimal_step\": \"100\", \"start_price\": \"10000\", \"start\": null, \"duration\": \"60000000000\", \"buy_out_price\": \"10000000000\", \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 1
```

first auction bid comes
```bash
near call $MARKET_CONTRACT_ID auction_add_bid '{"auction_id": "0", "token_type": "near"}' --accountId $ALICE --depositYocto 10300
```

check auctions
```bash
near view $MARKET_CONTRACT_ID get_auction_json '{"auction_id": "0"}'

near view $MARKET_CONTRACT_ID get_auction_json '{"auction_id": "1"}'
```

cancel second auction and finish first
```bash
near call $MARKET_CONTRACT_ID cancel_auction '{"auction_id": "1"}' --accountId $CONTRACT_PARENT --depositYocto 1

near call $MARKET_CONTRACT_ID finish_auction '{"auction_id": "0"}' --accountId $ALICE --gas 200000000000000
```

get our not used deposits back
```bash
near call $MARKET_CONTRACT_ID storage_withdraw --accountId $CONTRACT_PARENT --depositYocto 1
```