mod nft_core;
mod token;
mod payouts;

use std::collections::HashMap;

use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NFT_METADATA_SPEC,
};
use near_sdk::{require, AccountId, PanicOnDefault, Balance};

use near_contract_standards::non_fungible_token::{metadata::TokenMetadata, TokenId};
use near_contract_standards::non_fungible_token::{NonFungibleToken, Token};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupSet, UnorderedSet, UnorderedMap};
use near_sdk::{env, near_bindgen, BorshStorageKey};

type TokenSeriesId = String;
pub const TOKEN_DELIMETER: char = ':';

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenSeries {
	metadata: TokenMetadata,
	creator_id: AccountId,
	tokens: UnorderedSet<TokenId>,
    price: Option<Balance>,
    is_mintable: bool,
    royalty: HashMap<AccountId, u32>
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
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata, private_minters: Vec<AccountId>) -> Self {
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
}
