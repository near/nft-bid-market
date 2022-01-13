use crate::token_series::TokenSeriesJson;
use crate::*;

impl Nft {
    pub fn nft_series(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<TokenSeriesJson> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            (self.token_series_by_id.len() as u128) > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");

        self.token_series_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(token_series_id, token_series)| TokenSeriesJson {
                token_series_id,
                metadata: token_series.metadata,
                creator_id: token_series.creator_id,
                royalty: token_series.royalty,
            })
            .collect()
    }

    pub fn nft_supply_for_series(&self, token_series_id: TokenSeriesId) -> U128 {
        U128::from(
            self.token_series_by_id
                .get(&token_series_id)
                .unwrap_or_else(|| env::panic_str("Could not find token series"))
                .tokens
                .len() as u128,
        )
    }
}
