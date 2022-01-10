mod nft_core;
mod payouts;
mod token;

use std::collections::HashMap;

use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{metadata::TokenMetadata, TokenId};
use near_contract_standards::non_fungible_token::{NonFungibleToken, Token, refund_deposit};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupSet, UnorderedMap, UnorderedSet};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, BorshStorageKey};
use near_sdk::{require, AccountId, Balance, PanicOnDefault};

use crate::payouts::MAXIMUM_ROYALTY;

type TokenSeriesId = String;
pub const TOKEN_DELIMETER: char = ':';


#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenSeries {
    metadata: TokenMetadata,
    creator_id: AccountId,
    tokens: UnorderedSet<TokenId>,
    price: Option<Balance>,
    is_mintable: bool,
    royalty: HashMap<AccountId, u32>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenSeriesJson {
    token_series_id: TokenSeriesId,
    metadata: TokenMetadata,
    creator_id: AccountId,
    royalty: HashMap<AccountId, u32>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Nft {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,

    private_minters: LookupSet<AccountId>,
    token_series_by_id: UnorderedMap<TokenSeriesId, TokenSeries>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,

    TokensBySeriesInner { token_series: String },
}

#[near_bindgen]
impl Nft {
    #[init]
    pub fn new_default_meta(owner_id: AccountId) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "Example NEAR non-fungible token".to_string(),
                symbol: "EXAMPLE".to_string(),
                icon: None,
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
            Default::default(),
        )
    }

    #[init]
    pub fn new(
        owner_id: AccountId,
        metadata: NFTContractMetadata,
        private_minters: Vec<AccountId>,
    ) -> Self {
        require!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut minters = LookupSet::new(b"m");
        minters.extend(private_minters);
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            private_minters: minters,
            token_series_by_id: UnorderedMap::new(b"s"),
        }
    }

    // private method
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        token_owner_id: AccountId,
        token_metadata: TokenMetadata,
    ) -> Token {
        assert_eq!(
            env::predecessor_account_id(),
            self.tokens.owner_id,
            "Unauthorized"
        );
        self.tokens
            .internal_mint(token_id, token_owner_id, Some(token_metadata))
    }

    #[payable]
    pub fn nft_create_series(
        &mut self,
        token_metadata: TokenMetadata,
        price: Option<U128>,
        royalty: Option<HashMap<AccountId, u32>>,
    ) -> TokenSeriesJson {
        let initial_storage_usage = env::storage_usage();
        let token_series_id = (self.token_series_by_id.len() + 1).to_string();
        require!(token_metadata.title.is_some());
        let mut total_payouts = 0;
        let royalty_res: HashMap<AccountId, u32> = if let Some(royalty) = royalty {
            total_payouts = royalty.values().sum();
            royalty
        } else {
            HashMap::new()
        };
        require!(total_payouts <= MAXIMUM_ROYALTY, format!("exceeds maximum royalty {}", MAXIMUM_ROYALTY));
        let price: Option<u128> = if price.is_some() {
            Some(price.unwrap().0)
        } else {
            None
        };

        self.token_series_by_id.insert(&token_series_id, &TokenSeries{
            metadata: token_metadata.clone(),
            creator_id: env::predecessor_account_id(),
            tokens: UnorderedSet::new(
                StorageKey::TokensBySeriesInner {
                    token_series: token_series_id.clone(),
                }
                .try_to_vec()
                .unwrap(),
            ),
            price,
            is_mintable: true,
            royalty: royalty_res.clone(),
        });

        refund_deposit(env::storage_usage() - initial_storage_usage);
        
        TokenSeriesJson {
            token_series_id,
            metadata: token_metadata,
            creator_id: env::predecessor_account_id(),
            royalty: royalty_res,
        }
    }
}
