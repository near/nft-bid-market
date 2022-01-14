#!/bin/bash
set -e
sh build-all.sh # building all

near dev-deploy res/nft_bid_market.wasm
source neardev/dev-account.env
cat neardev/dev-account.env > .env

CONTRACT_PARENT=$CONTRACT_NAME

MARKET_CONTRACT_ID=$CONTRACT_PARENT
NFT_CONTRACT_ID=nft.$CONTRACT_PARENT

set -e
near create-account $NFT_CONTRACT_ID --masterAccount $CONTRACT_PARENT --initialBalance "27"
# Set up
near deploy $NFT_CONTRACT_ID --wasmFile res/nft_contract.wasm

# inits
near call $NFT_CONTRACT_ID new_default_meta '{"owner_id": "'$CONTRACT_PARENT'", "market_id": "'$MARKET_CONTRACT_ID'"}' --accountId $NFT_CONTRACT_ID
near call $MARKET_CONTRACT_ID new '{"nft_ids": ["'$NFT_CONTRACT_ID'"], "owner_id": "'$CONTRACT_PARENT'"}' --accountId $MARKET_CONTRACT_ID
# create token with some metadata

# CONTRACT_PARENT creates 3 token
near call $NFT_CONTRACT_ID nft_create_series '{"token_metadata": {"title": "some title", "media": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/00/Olympus_Mons_alt.jpg/1024px-Olympus_Mons_alt.jpg", "copies": 3}, "royalty": {"'$CONTRACT_PARENT'": 500}}' --accountId $CONTRACT_PARENT --deposit 0.005 # deposit is to cover storage
# so he can mint let's say 2 tokens to himself
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1 # only creator can mint his own tokens
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1

near call $MARKET_CONTRACT_ID storage_deposit --accountId $CONTRACT_PARENT --deposit 0.01 # cover storage of sale

near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:1", "account_id": "'$MARKET_CONTRACT_ID'", "msg": "{\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"is_auction\": false, \"start\": null, \"end\": null }"}' --accountId $CONTRACT_PARENT --deposit 1

near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:1"}' --accountId $NFT_CONTRACT_ID --depositYocto 10000 --gas 200000000000000