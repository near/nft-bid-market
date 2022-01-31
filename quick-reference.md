# NFT bid market

NFT bid market consists of two contracts: NFT and Market.

NFT contract allows to create and manage a token or token series. 
It supports Metadata, Approval Management and Royalties.

Market contract handles sales, bids and auctions.

To build both contracts and deploy it on dev account:
```bash
sh deploy-testnet.sh
source .env
```

Now we have `CONTRACT_PARENT` and three subaccounts: `MARKET_CONTRACT_ID`, `NFT_CONTRACT_ID` and `ALICE` ready to go.

Initialize contracts:
```bash
near call $NFT_CONTRACT_ID new_default_meta '{"owner_id": "'$CONTRACT_PARENT'", "market_id": "'$MARKET_CONTRACT_ID'"}' --accountId $NFT_CONTRACT_ID
near call $MARKET_CONTRACT_ID new '{"nft_ids": ["'$NFT_CONTRACT_ID'"], "owner_id": "'$CONTRACT_PARENT'"}' --accountId $MARKET_CONTRACT_ID
```

## Market contract

We can create either a new sale or a new auction.

### Sale

`CONTRACT_PARENT` creates the series of maximum seven tokens:
```bash
near call $NFT_CONTRACT_ID nft_create_series '{"token_metadata": {"title": "some title", "media": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/00/Olympus_Mons_alt.jpg/1024px-Olympus_Mons_alt.jpg", "copies": 7}, "royalty": {"'$CONTRACT_PARENT'": 500}}' --accountId $CONTRACT_PARENT --deposit 0.005
```
And mints three of them:
```bash
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1
```
Now `NFT_CONTRACT_ID` has three tokens with ids `1:1`, `1:2` and `1:3`.

Before creating a sale the user needs to cover the storage (0.01 per one sale):
```bash
near call $MARKET_CONTRACT_ID storage_deposit --accountId $CONTRACT_PARENT --deposit 0.1
```

`CONTRACT_PARENT` puts all three tokens on sale:
```bash
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:1", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Sale\": {\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"start\": null, \"end\": null, \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 1

near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:2", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Sale\": {\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"start\": null, \"end\": null, \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 1

near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:3", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Sale\": {\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"start\": null, \"end\": null, \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 1
```
He sets the price of `10000` yoctoNEAR for each token. Fees are automatically added to this amount.

The seller can withdraw any extra storage deposits (will return 0.07 in this case)
```bash
near call $MARKET_CONTRACT_ID storage_withdraw --accountId $CONTRACT_PARENT --depositYocto 1
```

Now any other account (in our case it is `ALICE`) can buy or offer to buy any of these tokens. 
The difference is in the deposit which she attaches to `offer`. 
If the attached deposit is equal to the price, she automatically buys it. The price is now equal to `10300` since a protocol fee (3%) was added.
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:1"}' --accountId $ALICE --depositYocto 10300 --gas 200000000000000
```

If `ALICE` tries to buy the second token (`1:2`), but the attached deposit less than the required price, she will only offer to buy the token.
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:2"}' --accountId $ALICE --depositYocto 8000 --gas 200000000000000

near call $MARKET_CONTRACT_ID accept_offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:2", "ft_token_id": "near"}' --accountId $CONTRACT_PARENT --gas 200000000000000
```
`ALICE` gets the token only after `CONTRACT_PARENT` accepts the offer using `accept_offer`.

If `CONTRACT_PARENT` wants to increase or decrease the price of `1:3`, he can run 
```bash
near call $MARKET_CONTRACT_ID update_price '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:3", "ft_token_id": "near", "price": "12000"}' --accountId $CONTRACT_PARENT --depositYocto 1
```
Now the price is `12360` yoctoNEAR (`360` yoctoNEAR added as the protocol fee), so if `ALICE` tries to buy it at a price `10300` yoctoNEAR, she won't get it automatically and will need to wait for `CONTRACT_PARENT` to accept the offer.

If `ALICE` adds a bid and then decides to remove it she could spend 1 yoctoNEAR calling `remove_bid`:
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:3"}' --accountId $ALICE --depositYocto 10000 --gas 200000000000000

near call $MARKET_CONTRACT_ID remove_bid '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:3", "bid": {"owner_id": "'$ALICE'", "price": "10000", "origins": {}}}' --accountId $ALICE --depositYocto 1
```
This would remove her bid and return her money.

The sale can be removed by `CONTRACT_PARENT` (and he would also need to pay 1 yoctoNEAR for this action):
```bash
near call $MARKET_CONTRACT_ID remove_sale '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:3"}' --accountId $CONTRACT_PARENT --depositYocto 1
```
This removes the sale and corresponding bids and returns the money.

#### View methods for sales
To find number of sales:
```bash
near view $MARKET_CONTRACT_ID get_supply_sales
```

To find number of sales for given owner:
```bash
near view $MARKET_CONTRACT_ID get_supply_by_owner_id '{"account_id": "'$CONTRACT_PARENT'"}'
```

To get sales for the given owner:
```bash
near view $MARKET_CONTRACT_ID get_sales_by_owner_id '{"account_id": "'$CONTRACT_PARENT'", "from_index": "0", "limit": 10}'
```

To find number of sales for given nft contract:
```bash
near view $MARKET_CONTRACT_ID get_supply_by_nft_contract_id '{"nft_contract_id": "'$NFT_CONTRACT_ID'"}'
```

To get sales for the given nft contract:
```bash
near view $MARKET_CONTRACT_ID get_sales_by_nft_contract_id '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "from_index": "0", "limit": 10}'
```

To find number of sales for token type:
```bash
near view $MARKET_CONTRACT_ID get_supply_by_nft_token_type '{"token_type": "near"}'
```

To get sales for token type:
```bash
near view $MARKET_CONTRACT_ID get_sales_by_nft_token_type '{"token_type": "near", "from_index": "0", "limit": 10}'
```

To get the sale:
```bash
near view $MARKET_CONTRACT_ID get_sale '{"nft_contract_token": "'$NFT_CONTRACT_ID'||1:3"}'
```

### Auction

`CONTRACT_PARENT` mints two more tokens `1:4` and `1:5`:
```bash
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1
```

And puts them on auction:
```bash
near call $MARKET_CONTRACT_ID storage_deposit --accountId $CONTRACT_PARENT --deposit 0.02

near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:4", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Auction\": {\"token_type\": \"near\", \"minimal_step\": \"100\", \"start_price\": \"10000\", \"start\": null, \"duration\": \"900000000000\", \"buy_out_price\": \"10000000000\", \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:5", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Auction\": {\"token_type\": \"near\", \"minimal_step\": \"100\", \"start_price\": \"10000\", \"start\": null, \"duration\": \"900000000000\", \"buy_out_price\": \"10000000000\", \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 1
```
He set the minimal price to `10000` and minimal step `1000`. Everyone should see the price as `10300` and minimal step `1030` (because it includes protocol fee). The duration `900000000000` corresponds to 15 minutes. You can't set the duration lower than that. One can set the specific start time, otherwise the auction starts as soon as the command is run. He also specified the `buy_out_price`, meaning that anyone can buy the token by this price.

`CONTRACT_PARENT` can cancel his auction beforehand in case there is no bid:
```bash
near call $MARKET_CONTRACT_ID cancel_auction '{"auction_id": "1"}' --accountId $CONTRACT_PARENT --depositYocto 1
```

`ALICE` can create a bid on the ongoing auction:
```bash
near call $MARKET_CONTRACT_ID auction_add_bid '{"auction_id": "0", "token_type": "near"}' --accountId $ALICE --depositYocto 10300
```
In our case, this call happens less than 15 minutes before the end of the auction, thus the auction is extended.

If `ALICE` had called `auction_add_bid` with deposit more or equal to `buy_out_price`, she would have automatically bought it. In this case the auction would have ended ahead of time.

After auction ends anyone can finish it:
```bash
near call $MARKET_CONTRACT_ID finish_auction '{"auction_id": "0"}' --accountId $ALICE --gas 200000000000000
```

#### View methods for auctions

To get the creator of the latest bid:
```bash
near view $MARKET_CONTRACT_ID get_current_buyer '{"auction_id": "0"}'
```

To check whether the auction in progress:
```bash
near view $MARKET_CONTRACT_ID check_auction_in_progress '{"auction_id": "0"}'
```

To get the auction:
```bash
near view $MARKET_CONTRACT_ID get_auction_json '{"auction_id": "0"}'
```

To get the minimal bid one could bid (including fees):
```bash
near view $MARKET_CONTRACT_ID get_minimal_next_bid '{"auction_id": "0"}'
```

To get the amount of the latest bid:
```bash
near view $MARKET_CONTRACT_ID get_current_bid '{"auction_id": "0"}'
```

## NFT contract

Owner can assign private minters
```bash
near call $NFT_CONTRACT_ID add_private_minter '{"account_id": "'$ALICE'"}' --accountId $CONTRACT_PARENT
```

Owner of series can approve market for minting tokens in this series
```bash
near call $NFT_CONTRACT_ID nft_series_market_approve '{"token_series_id": "1", "sale_conditions": {"near": "1200"}, "copies": 1, "approved_market_id": "'$MARKET_CONTRACT_ID'"}' --accountId $CONTRACT_PARENT --deposit 1
```
This method will call **approved_market_id**'s method **nft_on_series_approve** with arguments 
```bash
'{"token_series": {"sale_conditions": {"near": "1200"}, "series_id": "1", "owner_id": "'$CONTRACT_PARENT'", "copies": 1}}'
```

### View methods on nft token series

Get metadata, owner_id and royalty of specific token series
```bash
near view $NFT_CONTRACT_ID nft_get_series_json '{"token_series_id": "1"}'
```

Get how many tokens of the specific token series already minted
```bash
near view $NFT_CONTRACT_ID nft_supply_for_series '{"token_series_id": "1"}'
```

Get a list of all series
```bash
near view $NFT_CONTRACT_ID nft_series
```
or with pagination
```bash
near view $NFT_CONTRACT_ID nft_series '{"from_index": "0", "limit": 10}'
```