# Our vision of buyer side fee, without orders:
Just add to the price fee %'s, and subtract fee from both sides, example:
- we have protocol fee 3%
- **A** approves to sell ‘some_nft’ for 100N, we put it on sale for 103N
- some_nft->103N
- **B** buys it for 103N
- We subtract from 103% 3%, from buyer, that’s 3N
- After that we subtract another 3% from seller, and 97N goes to royalties and seller
- Similar goes for auctions(we add 3% to min_step, buy_out_proce...)
