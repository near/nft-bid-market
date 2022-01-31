mod nft_core;
mod token;

pub mod common;
mod series_views;
pub mod event;
use common::*;

mod token_series;
use event::NearEvent;
use near_contract_standards::non_fungible_token::refund_deposit_to_account;
use near_sdk::{ext_contract, Gas, Promise};
use payouts::assert_at_least_one_yocto;
use token_series::{TokenSeries, TokenSeriesId, TokenSeriesSale, TOKEN_DELIMETER};

mod payouts;
use crate::{payouts::MAXIMUM_ROYALTY, event::NftMintData};

use std::collections::HashMap;

const GAS_FOR_NFT_APPROVE: Gas = Gas(10_000_000_000_000);

// Since Near doesn't support multitoken(yet) by default we need to create some workaround
// In this nft implementation every token is part of TokenSeries
// Token series is tokens, that share same metadata.
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Nft {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,

    private_minters: LookupSet<AccountId>,
    token_series_by_id: UnorderedMap<TokenSeriesId, TokenSeries>,
    market_id: AccountId,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,

    TokensBySeriesInner { token_series: String },
    TokensPerOwner { account_hash: Vec<u8> },
}

#[near_bindgen]
impl Nft {
    #[init]
    pub fn new_default_meta(owner_id: AccountId, market_id: AccountId) -> Self {
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
            market_id,
        )
    }

    #[init]
    pub fn new(
        owner_id: AccountId,
        metadata: NFTContractMetadata,
        private_minters: Vec<AccountId>,
        market_id: AccountId,
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
            market_id,
        }
    }

    // public mint,
    // mints NFT with metadata of token series
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_series_id: TokenSeriesId,
        reciever_id: AccountId,
        refund_id: Option<AccountId>,
    ) -> TokenId {
        let refund_id = refund_id.unwrap_or_else(env::predecessor_account_id);
        let initial_storage_usage = env::storage_usage();

        let mut token_series = self
            .token_series_by_id
            .get(&token_series_id)
            .expect("Token series does not exist");
        require!(
            env::predecessor_account_id().eq(&token_series.owner_id)
                || if let Some(ref approved_market_id) = token_series.approved_market_id {
                    env::predecessor_account_id().eq(approved_market_id)
                } else {
                    false
                },
            "permission denied"
        );
        require!(
            token_series.tokens.len() < token_series.metadata.copies.unwrap_or(u64::MAX),
            "Max token minted"
        );
        let token_id = format!(
            "{}{}{}",
            token_series_id,
            TOKEN_DELIMETER,
            token_series.tokens.len() + 1
        );
        let metadata = TokenMetadata {
            title: None,       // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
            description: None, // free-form description
            media: None, // URL to associated media, preferably to decentralized, content-addressed storage
            media_hash: None, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
            copies: None, // number of copies of this set of metadata in existence when token was minted.
            issued_at: Some(env::block_timestamp().to_string()), // ISO 8601 datetime when token was issued or minted
            expires_at: None,     // ISO 8601 datetime when token expires
            starts_at: None,      // ISO 8601 datetime when token starts being valid
            updated_at: None,     // ISO 8601 datetime when token was last updated
            extra: None, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
            reference: None, // URL to an off-chain JSON file with more info.
            reference_hash: None, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
        };

        // implementation from NonFungibleToken::internal_mint_with_refund()
        // Core behavior: every token must have an owner
        self.tokens.owner_by_id.insert(&token_id, &reciever_id);
        // Metadata extension: Save metadata, keep variable around to return later.
        // Note that check above already panicked if metadata extension in use but no metadata
        // provided to call.
        self.tokens
            .token_metadata_by_id
            .as_mut()
            .and_then(|by_id| by_id.insert(&token_id, &metadata));

        // Enumeration extension: Record tokens_per_owner for use with enumeration view methods.
        if let Some(tokens_per_owner) = &mut self.tokens.tokens_per_owner {
            let mut token_ids = tokens_per_owner.get(&reciever_id).unwrap_or_else(|| {
                UnorderedSet::new(StorageKey::TokensPerOwner {
                    account_hash: env::sha256(reciever_id.as_bytes()),
                })
            });
            token_ids.insert(&token_id);
            tokens_per_owner.insert(&reciever_id, &token_ids);
        }
        token_series.tokens.insert(&token_id);
        self.token_series_by_id
            .insert(&token_series_id, &token_series);

        refund_deposit_to_account(env::storage_usage() - initial_storage_usage, refund_id);
        
        // Event
        let mint_log = NftMintData::new(&reciever_id, vec![&token_id], None);
        NearEvent::nft_mint(vec![mint_log]).emit();
        
        token_id
    }

    // Create series with given metadata and royalty
    #[payable]
    pub fn nft_create_series(
        &mut self,
        token_metadata: TokenMetadata,
        royalty: Option<HashMap<AccountId, u32>>,
    ) -> TokenSeriesId {
        let initial_storage_usage = env::storage_usage();
        let owner_id = env::predecessor_account_id();
        let token_series_id = (self.token_series_by_id.len() + 1).to_string();
        require!(
            token_metadata.title.is_some(),
            "title is missing from token metadata"
        );
        let mut total_payouts = 0;
        let royalty_res: HashMap<AccountId, u32> = if let Some(royalty) = royalty {
            total_payouts = royalty.values().sum();
            royalty
        } else {
            HashMap::new()
        };
        require!(
            total_payouts <= MAXIMUM_ROYALTY,
            format!("maximum royalty cap exceeded {}", MAXIMUM_ROYALTY)
        );

        self.token_series_by_id.insert(
            &token_series_id,
            &TokenSeries {
                metadata: token_metadata,
                owner_id,
                tokens: UnorderedSet::new(
                    StorageKey::TokensBySeriesInner {
                        token_series: token_series_id.clone(),
                    }
                    .try_to_vec()
                    .unwrap(),
                ),
                royalty: royalty_res,
                approved_market_id: None,
            },
        );

        refund_deposit(env::storage_usage() - initial_storage_usage);

        token_series_id
    }

    pub fn add_private_minter(&mut self, account_id: AccountId) {
        require!(env::predecessor_account_id().eq(&self.tokens.owner_id));
        self.private_minters.insert(&account_id);
    }

    #[payable]
    pub fn nft_series_market_approve(
        &mut self,
        token_series_id: TokenId,
        sale_conditions: token_series::SaleConditions,
        copies: u64,
        approved_market_id: AccountId,
    ) -> Promise {
        let initial_storage_usage = env::storage_usage();
        let mut token_series = self
            .token_series_by_id
            .get(&token_series_id)
            .expect("Series not found");
        require!(
            env::predecessor_account_id().eq(&token_series.owner_id),
            "Not token owner"
        );
        require!(
            token_series.metadata.copies.unwrap_or(u64::MAX) - token_series.tokens.len() >= copies,
            "Too many copies"
        );
        token_series.approved_market_id = Some(approved_market_id.clone());
        self.token_series_by_id
            .insert(&token_series_id, &token_series);
        refund_deposit(env::storage_usage() - initial_storage_usage);
        ext_contract::nft_on_series_approve(
            TokenSeriesSale {
                sale_conditions,
                series_id: token_series_id,
                owner_id: token_series.owner_id,
                copies,
            },
            approved_market_id,
            0,
            env::prepaid_gas() - GAS_FOR_NFT_APPROVE,
        )
    }
    // TODO:

    // private minting
    // pub fn private_mint()
}

near_contract_standards::impl_non_fungible_token_approval!(Nft, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Nft, tokens);

#[ext_contract(ext_contract)]
trait ExtContract {
    fn nft_on_series_approve(&mut self, token_series: TokenSeriesSale);
}
