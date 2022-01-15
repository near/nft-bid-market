#!/bin/bash
rm -rvf res

set -e
sh build-all.sh # building all

rm -f .env
rm -rvf neardev

near dev-deploy res/nft_bid_market.wasm
source neardev/dev-account.env
cat neardev/dev-account.env > .env

CONTRACT_PARENT=$CONTRACT_NAME
MARKET_CONTRACT_ID=$CONTRACT_PARENT
NFT_CONTRACT_ID=nft.$CONTRACT_PARENT
ALICE=alice.$CONTRACT_PARENT

set -e
near create-account $NFT_CONTRACT_ID --masterAccount $CONTRACT_PARENT --initialBalance "27"
near create-account $ALICE --masterAccount $CONTRACT_PARENT --initialBalance 20

# Set up
near deploy $NFT_CONTRACT_ID --wasmFile res/nft_contract.wasm
