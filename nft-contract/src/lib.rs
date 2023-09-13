use std::collections::HashMap;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen, AccountId, Balance, CryptoHash, PanicOnDefault, Promise, PromiseOrValue,
};

use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::approval::*;
pub use crate::royalty::*;
pub use crate::events::*;

mod internal;
mod approval; 
mod enumeration; 
mod metadata; 
mod mint; 
mod nft_core; 
mod royalty; 
mod events;

/// NFT 표준의 버전으로 사용됩니다.
pub const NFT_METADATA_SPEC: &str = "1.0.0";
/// 우리가 사용하는 NFT 표준의 이름입니다.
pub const NFT_STANDARD_NAME: &str = "nep171";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    // 계약의 소유자
    pub owner_id: AccountId,

    // 주어진 계정에 대한 모든 토큰 ID를 추적합니다.
    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,

    // 주어진 토큰 ID에 대한 토큰 구조를 추적합니다.
    pub tokens_by_id: LookupMap<TokenId, Token>,

    // 주어진 토큰 ID에 대한 토큰 메타데이터를 추적합니다.
    pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,

    // 계약의 메타데이터를 추적합니다.
    pub metadata: LazyOption<NFTContractMetadata>,
}

/// 영구 컬렉션의 키를 위한 도우미 구조체입니다.
#[derive(BorshSerialize)]
pub enum StorageKey {
    TokensPerOwner,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
    TokensById,
    TokenMetadataById,
    NFTContractMetadata,
    TokensPerType,
    TokensPerTypeInner { token_type_hash: CryptoHash },
    TokenTypesLocked,
}

#[near_bindgen]
impl Contract {
    /*
        초기화 함수입니다 (단 한 번만 호출 가능).
        이 함수는 사용자가 수동으로 메타데이터를 입력하지 않아도 되도록
        계약을 기본 메타데이터로 초기화합니다.
    */
    #[init]
    pub fn new_default_meta(owner_id: AccountId) -> Self {
        // owner_id로 "new" 함수를 호출하고 기본 메타데이터를 전달합니다.
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: "nft-1.0.0".to_string(),
                name: "NFT Tutorial Contract".to_string(),
                symbol: "Linktown-portfolio".to_string(),
                icon: None,
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }

    /*
        초기화 함수입니다 (단 한 번만 호출 가능).
        이 함수는 전달된 메타데이터와 owner_id로 계약을 초기화합니다.
    */
    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
        // 모든 필드를 초기화한 Self 타입의 변수를 생성합니다.
        let this = Self {
            // 저장소 키는 컬렉션의 접두사입니다. 이는 데이터 충돌을 방지하는데 도움이 됩니다.
            tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner.try_to_vec().unwrap()),
            tokens_by_id: LookupMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            token_metadata_by_id: UnorderedMap::new(
                StorageKey::TokenMetadataById.try_to_vec().unwrap(),
            ),
            // owner_id 필드를 전달된 owner_id로 설정합니다.
            owner_id,
            metadata: LazyOption::new(
                StorageKey::NFTContractMetadata.try_to_vec().unwrap(),
                Some(&metadata),
            ),
        };

        // Contract 객체를 반환합니다.
        this
    }
}

#[cfg(test)]
mod tests;
