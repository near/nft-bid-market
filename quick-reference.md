# NFT bid market

NFT bid market consists of two contracts: NFT and Market.

NFT contract allows to create and manage a token or token series. 
It supports Metadata, Approval Management and Royalties [standards](https://nomicon.io/Standards/NonFungibleToken/README.html).

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

## NFT contract

NFT contract supports [standards](https://nomicon.io/Standards/NonFungibleToken/README.html) for Metadata, Approval Management and Royalties. In addition, it manages private minting and allows the owner of the series to give a permission to the market to mint tokens.

In order to put a token or series of tokens on a sale or auction, one must create and mint it first.

`CONTRACT_PARENT` creates the series of maximum seven tokens:
```bash
near call $NFT_CONTRACT_ID nft_create_series '{"token_metadata": {"title": "some title", "media": "https://upload.wikimedia.org/wikipedia/commons/thumb/0/00/Olympus_Mons_alt.jpg/1024px-Olympus_Mons_alt.jpg", "copies": 7}, "royalty": {"'$CONTRACT_PARENT'": 500}}' --accountId $CONTRACT_PARENT --deposit 0.005
```
And mints five of them:
```bash
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $CONTRACT_PARENT --deposit 1
```
Now there are five tokens with ids `1:1`, `1:2`, `1:3`, `1:4` and `1:5`.

`CONTRACT_PARENT` (the owner) can assign private minters
```bash
near call $NFT_CONTRACT_ID add_private_minter '{"account_id": "'$ALICE'"}' --accountId $CONTRACT_PARENT
```

The owner of the series can approve market for minting tokens in this series
```bash
near call $MARKET_CONTRACT_ID storage_deposit --accountId $CONTRACT_PARENT --deposit 0.01

near call $NFT_CONTRACT_ID nft_series_market_approve '{"token_series_id": "1", "sale_conditions": {"near": "1200"}, "copies": 1, "approved_market_id": "'$MARKET_CONTRACT_ID'"}' --accountId $CONTRACT_PARENT --deposit 1
```
This method will call **approved_market_id**'s method **nft_on_series_approve** with arguments 
```bash
'{"token_series": {"sale_conditions": {"near": "1200"}, "series_id": "1", "owner_id": "'$CONTRACT_PARENT'", "copies": 1}}'
```
After this `MARKET_CONTRACT_ID` can mint a new token `1:6`:
```bash
near call $NFT_CONTRACT_ID nft_mint '{"token_series_id": "1", "reciever_id": "'$CONTRACT_PARENT'"}' --accountId $MARKET_CONTRACT_ID --deposit 1
```

### List of view methods for nft token series

To get metadata, owner_id and royalty of specific token series:
```bash
near view $NFT_CONTRACT_ID nft_get_series_json '{"token_series_id": "1"}'
```

To get how many tokens of the specific token series have already been minted:
```bash
near view $NFT_CONTRACT_ID nft_supply_for_series '{"token_series_id": "1"}'
```

To get a list of all series (with pagination or without it):
```bash
near view $NFT_CONTRACT_ID nft_series '{"from_index": "0", "limit": 10}'
near view $NFT_CONTRACT_ID nft_series
```

## Market contract

Using Market contract a user can put his NFT on a sale or an auction.
He specifies the conditions on which he wants to sell NFT, such as FT type and price, start and end (or duration for auction), origins.
Other users create bids, offering to buy (or buying) the NFT. Bids for sales can have start/end time.

### Workflow for creating and using sales

Before creating a sale the user needs to cover the storage (0.01 per one sale):
```bash
near call $MARKET_CONTRACT_ID storage_deposit --accountId $CONTRACT_PARENT --deposit 0.1
```

`CONTRACT_PARENT` puts tokens `1:1`, `1:2` and `1:3` on sale:
```bash
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:1", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Sale\": {\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"start\": null, \"end\": null, \"origins\": {\"'$NFT_CONTRACT_ID'\": 100}} }"}' --accountId $CONTRACT_PARENT --deposit 1

near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:2", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Sale\": {\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"start\": null, \"end\": null, \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 1

near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:3", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Sale\": {\"sale_conditions\": {\"near\": \"10000\"}, \"token_type\": \"1\", \"start\": null, \"end\": null, \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 1
```
Only the first token has origin fee. It might be paid to `NFT_CONTRACT_ID` after the NFT is sold. The number `100` in the method corresponds to 1% origin fee.

`CONTRACT_PARENT` sets the price of `10000` yoctoNEAR for each token. Fees are automatically added to this amount, thus if you look at the sale
```bash
near view $MARKET_CONTRACT_ID get_sale '{"nft_contract_token": "'$NFT_CONTRACT_ID'||1:1"}'
```
the `sale_conditions` field shows the price to be `10300` due to 3% protocol fee.

The seller can withdraw any extra storage deposits (will return 0.07 in this case)
```bash
near call $MARKET_CONTRACT_ID storage_withdraw --accountId $CONTRACT_PARENT --depositYocto 1
```

Any other account (in our case it is `ALICE`) can buy or offer to buy any of these tokens. 
The difference is in the deposit which she attaches to `offer`. 
If the attached deposit is equal to the price (`10300` including protocol fee), she automatically buys it.
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:1"}' --accountId $ALICE --depositYocto 10300 --gas 200000000000000
```

If `ALICE` tries to buy the second token (`1:2`), but the attached deposit less than the required price, she will only offer to buy the token.
```bash
near call $MARKET_CONTRACT_ID offer '{"nft_contract_id": "'$NFT_CONTRACT_ID'", "token_id": "1:2"}' --accountId $ALICE --depositYocto 10200 --gas 200000000000000

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

### List of view methods for sales
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

### Workflow for creating and using auction

`CONTRACT_PARENT` puts tokens `1:4` and `1:5` on auction:
```bash
near call $MARKET_CONTRACT_ID storage_deposit --accountId $CONTRACT_PARENT --deposit 0.02

near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:4", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Auction\": {\"token_type\": \"near\", \"minimal_step\": \"100\", \"start_price\": \"10000\", \"start\": null, \"duration\": \"900000000000\", \"buy_out_price\": \"10000000000\", \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 1
near call $NFT_CONTRACT_ID nft_approve '{"token_id": "1:5", "account_id": "'$MARKET_CONTRACT_ID'", 
"msg": "{\"Auction\": {\"token_type\": \"near\", \"minimal_step\": \"100\", \"start_price\": \"10000\", \"start\": null, \"duration\": \"900000000000\", \"buy_out_price\": \"10000000000\", \"origins\": null} }"}' --accountId $CONTRACT_PARENT --deposit 1
```
He set the minimal price to `10000` and minimal step `1000`. Everyone should see the price as `10300` and minimal step `1030` (because it includes protocol fee). The duration `900000000000` corresponds to 15 minutes. You can't set the duration lower than that. One can set the specific start time, otherwise the auction starts as soon as the command is run. He also specified the `buy_out_price`, meaning that anyone can buy the token by this price.

`CONTRACT_PARENT` can cancel his auction before the end in case there is no bid:
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

### List of view methods for auctions

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
