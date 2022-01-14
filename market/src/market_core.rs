//use near_contract_standards::non_fungible_token::approval::NonFungibleTokenApprovalReceiver;
use near_contract_standards::non_fungible_token::hash_account_id;
use crate::sale::DELIMETER;
//use crate::sale_views;
use crate::*;

pub trait NonFungibleTokenApprovalReceiver {
    fn nft_on_approve(
        &mut self, 
        token_id: TokenId, 
        owner_id: AccountId, 
        approval_id: u64, 
        msg: String
    );
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleArgs {
    pub sale_conditions: SaleConditions,
    pub token_type: TokenType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auction: Option<bool>,

    pub start: Option<u64>,
    pub end: Option<u64>,
}

#[near_bindgen]
impl NonFungibleTokenApprovalReceiver for Market {
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    ) {
        /*require!(
            self.non_fungible_token_account_ids.contains(&env::predecessor_account_id()),
            "Only supports the one non-fungible token contract"
        );
        match msg.as_str() {
            _ => ()
        }
        todo!()*/

        // enforce cross contract call and owner_id is signer

        let nft_contract_id = env::predecessor_account_id();
        let signer_id = env::signer_account_id();
        assert_ne!(
            nft_contract_id,
            signer_id,
            "nft_on_approve should only be called via cross-contract call"
        );
        assert_eq!(
            &owner_id.clone(),
            &signer_id,
            "owner_id should be signer_id"
        );

        // enforce signer's storage is enough to cover + 1 more sale 

        let storage_amount = self.storage_amount().0;
        let owner_paid_storage = self.market.storage_deposits.get(&signer_id).unwrap_or(0);
        let signer_storage_required = (self.get_supply_by_owner_id(signer_id).0 + 1) as u128 * storage_amount;
        assert!(
            owner_paid_storage >= signer_storage_required,
            "Insufficient storage paid: {}, for {} sales at {} rate of per sale",
            owner_paid_storage, signer_storage_required / STORAGE_PER_SALE, STORAGE_PER_SALE
        );

        let SaleArgs { sale_conditions, token_type, is_auction, start, end } =
            near_sdk::serde_json::from_str(&msg).expect("Not valid SaleArgs");

        
        for (ft_token_id, _price) in sale_conditions.clone() {
            if !self.market.ft_token_ids.contains(&ft_token_id) {
                env::panic_str(
                    &format!("Token {} not supported by this market", ft_token_id),
                );
            }
        }

        // env::log(format!("add_sale for owner: {}", &owner_id).as_bytes());

        let bids = HashMap::new();

        let contract_and_token_id = format!("{}{}{}", nft_contract_id, DELIMETER, token_id);
        self.market.sales.insert(
            &contract_and_token_id,
            &Sale {
                owner_id: owner_id.clone(),
                approval_id,
                nft_contract_id: nft_contract_id.to_string(),
                token_id: token_id.clone(),
                sale_conditions,
                bids,
                created_at: env::block_timestamp()/1000000,
                token_type: token_type.clone(),
                is_auction,
                start,
                end,
            },
        );

        // extra for views

        let mut by_owner_id = self.market.by_owner_id.get(&owner_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::ByOwnerIdInner {
                    account_id_hash: hash_account_id(&owner_id),
                }
                .try_to_vec()
                .unwrap(),
            )
        });

        let owner_occupied_storage = u128::from(by_owner_id.len()) * STORAGE_PER_SALE;
        assert!(
            owner_paid_storage > owner_occupied_storage,
            "User has more sales than storage paid"
        );
        by_owner_id.insert(&contract_and_token_id);
        self.market.by_owner_id.insert(&owner_id, &by_owner_id);

        let mut by_nft_contract_id = self
            .market
            .by_nft_contract_id
            .get(&nft_contract_id)
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKey::ByNFTContractIdInner {
                        account_id_hash: hash_account_id(&nft_contract_id),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });
        by_nft_contract_id.insert(&token_id);
        self.market.by_nft_contract_id
            .insert(&nft_contract_id, &by_nft_contract_id);

        if let Some(token_type) = token_type {
            assert!(token_id.contains(&token_type), "TokenType should be substr of TokenId");
            let token_type = AccountId::new_unchecked(token_type);
            let mut by_nft_token_type = self
                .market
                .by_nft_token_type
                .get(&token_type)
                .unwrap_or_else(|| {
                    UnorderedSet::new(
                        StorageKey::ByNFTTokenTypeInner {
                            token_type_hash: hash_account_id(&token_type),
                        }
                        .try_to_vec()
                        .unwrap(),
                    )
                });
                by_nft_token_type.insert(&contract_and_token_id);
            self.market
                .by_nft_token_type
                .insert(&token_type, &by_nft_token_type);
        }
    }
}
