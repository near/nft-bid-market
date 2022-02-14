# Market

## market_core

### nft_on_approve
- Can only be called via cross-contract call
- `owner_id` must be the signer
- Panics if `owner_id` didn't pay for one more sale/auction
- Panics if the given `ft_token_id` is not supported by the market
- Start time is set to `block_timestamp` if it is not specified explicitly
- Creates a new sale/auction
### nft_on_series_approve
- Can only be called via cross-contract call
- `owner_id` must be the signer
- Panics if `owner_id` didn't pay for one more sale/auction
- Panics if the given `ft_token_id` is not supported by the market

## sale

### offer
- Should panic if there is no sale with given `contract_and_token_id`
- Should panic if the sale is not in progress
- Should panic if the NFT owner tries to make a bid on his own sale
- Should panic if the deposit equal to 0
- If the `attached_deposit` is equal to the price + fees, the purchase should be made. NFT is transferred to the buyer, ft transferred to the previous owner, protocol and origins fees are paid, the previous owner also pays royalty. The sale is removed from list of sales
- If the `attached_deposit` is not equal to the price + fees, a new bid should be added
### accept_offer
- Should panic if there is no sale with the given `nft_contract_id` and `token_id`
- Should panic if the sale is not in progress
- Should panic if there is no bids with given fungible token
- Should panic if the last bid is out of time
- The purchase should be made. NFT is transferred to the buyer, ft transferred to the previous owner, protocol and origins fees are paid, the previous owner also pays royalty. The sale is removed from list of sales
### update_price
- Should panic unless 1 yoctoNEAR is attached
- Should panic if there is no sale with the given `nft_contract_id` and `token_id`
- Should panic unless it is called by the creator of the sale
- Should panic if `ft_token_id` is not supported
- Changes the price
### remove_sale
- Should panic unless 1 yoctoNEAR is attached
- If the sale in progress, only the sale creator can remove the sale
- Refunds all bids

## bids

### add_bid
- Private method
- Should panic if `ft_token_id` is not supported
- Should panic if the `attached_deposit` less than the previous bid
### remove_bid
- Should panic unless 1 yoctoNEAR is attached
- Should panic if there is no sale with the given `nft_contract_id` and `token_id`
- Should panic if there is no bids with `ft_token_id`
- Refunds a bid, removes it from the list
### cancel_bid
- Should panic if the bid isn't finished yet
- Should panic if there is no sale with the given `nft_contract_id` and `token_id`
- Should panic if there is no bids with `ft_token_id`
- Refunds a bid, removes it from the list
### cancel_expired_bids
- Should panic if there is no sale with the given `nft_contract_id` and `token_id`
- Should panic if there is no bids with `ft_token_id`
- Refunds all expired bids, removes them from the list

## auctions

### auction_add_bid
- Should panic if `ft_token_id` is not supported
- Should panic if the auction is not in progress
- Should panic if the bid is smaller than the minimal bid
- Should panic if the bid is smaller than the previous one
- Extends an auction if the bid is added less than 15 minutes before the end
- The auction ends if the `attached_deposit` is bigger than the `buy_out_price` (plus fees)
### cancel_auction
- Should panic unless 1 yoctoNEAR is attached
- Can only be called by the creator of the auction
- Panics if auction is not active
- Panics if the auction already has a bid
- Removes the auction
### finish_auction
- Panics if auction is not active
- Should panic if called before the auction ends
- Panics if there is no bid
- The purchase should be made. NFT is transferred to the buyer, ft transferred to the previous owner, protocol and origins fees are paid, the previous owner also pays royalty. The auction is removed from list of auctions

## sale_views

### get_sale
### get_supply_sales
### get_sales
### get_supply_by_owner_id
### get_sales_by_owner_id
### get_supply_by_nft_contract_id
### get_sales_by_nft_contract_id
### get_supply_by_nft_token_type
### get_sales_by_nft_token_type

## auction_views

### get_auction_json
### get_auctions
### get_current_buyer
### get_current_bid
### check_auction_in_progress
### get_minimal_next_bid

# NFT

## lib

### nft_create_series
- Can only be called by the autorized account
- Panics if the title of the series is not specified
- Panics if the total royalty payout exceeds 50%
- Creates a new series with given metadata and royalty
- Refunds a deposit
### nft_mint
- Can only be called by the autorized account
- Panics if there is no series `token_series_id`
- Panics if the maximum number of tokens have already been minted
- Mints a new token
- Refunds a deposit
### nft_series_market_approve
- Panics if there is no series `token_series_id`
- Can only be called by the owner of the series
- Panics if the number of copies (including already minted tokens) exceeds the maximum number of copies
- Refunds a deposit
- Creates a cross contract call to `nft_on_series_approve`

## payouts

### nft_payout
- Panics if `token_id` contains `token_series_id`, which doesn't exist
- Panics if the number of royalties exceeds `max_len_payout`
- Panics if royalty exceeds 10000 yoctoNEAR?
- Splits the `balance` among royalties and owner, returns payout
### nft_transfer_payout
- Should panic unless 1 yoctoNEAR is attached
- Panics if `token_id` contains `token_series_id`, which doesn't exist
- Panics if the number of royalties exceeds `max_len_payout`
- Panics if invalid `memo` is provided
- Panics if total payout exceeds `ROYALTY_TOTAL_VALUE`
- Returns payout, which contains royalties and payouts from `memo`

## permissions

### grant
- Can only be called by the owner
- Adds a given account to the list of the autorized accounts
### deny
- Can only be called by the owner
- Removes a given account from the list of the autorized accounts
### set_private_minting
- Can only be called by the owner
- If `enabled` is true, turns on private minting
- If `enabled` is false, turns off private minting
### is_allowed
- Returns true if private minting is not enabled
- If private minting is enabled, returns whether an account is among private minters

## series_views

### nft_get_series_json
### nft_series
### nft_supply_for_series