//NFT 토큰과 그 소유자를 나열하는 방법이 포함되어 있습니다.
use crate::*;

#[near_bindgen]
impl Contract {
    //Query for the total supply of NFTs on the contract
    //계약에 있는 NFT의 전체 공급량을 쿼리합니다.
    //near view $ID nft_total_supply '{}'
    pub fn nft_total_supply(&self) -> U128 {
        //return the length of the token metadata by ID
        //ID 별 토큰 메타데이터의 길이를 반환합니다.
        U128(self.token_metadata_by_id.len() as u128)
    }

    //Query for nft tokens on the contract regardless of the owner using pagination
    //소유자와 상관없이 페이징을 사용하여 계약의 nft 토큰을 쿼리합니다.
    //near view $ID nft_tokens '{}'
    pub fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<JsonToken> {
        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        //페이징을 시작할 위치 - from_index가 있으면 그것을 사용하고, 그렇지 않으면 0 인덱스에서 시작합니다.
        let start = u128::from(from_index.unwrap_or(U128(0)));

        //iterate through each token using an iterator
        //반복자를 사용하여 각 토큰을 순회합니다.
        self.token_metadata_by_id.keys()
            //skip to the index we specified in the start variable
            //start 변수에 지정한 인덱스로 건너뜁니다.
            .skip(start as usize) 
            //take the first "limit" elements in the vector. If we didn't specify a limit, use 50
            //벡터에서 첫 번째 "limit" 요소를 가져옵니다. limit을 지정하지 않았다면 50을 사용합니다.
            .take(limit.unwrap_or(50) as usize) 
            //we'll map the token IDs which are strings into Json Tokens
            //문자열인 토큰 ID를 Json 토큰으로 매핑합니다.
            .map(|token_id| self.nft_token(token_id.clone()).unwrap())
            //since we turned the keys into an iterator, we need to turn it back into a vector to return
            //키를 반복자로 변환했으므로 반환하기 위해 벡터로 다시 변환해야 합니다.
            .collect()
    }

    //get the total supply of NFTs for a given owner
    //주어진 소유자에 대한 NFT의 전체 공급량을 가져옵니다. 
    //near view $ID nft_supply_for_owner '{"account_id":"buster_test.testnet"}' 
    pub fn nft_supply_for_owner(
        &self,
        account_id: AccountId,
    ) -> U128 {
        //get the set of tokens for the passed in owner
        //전달된 소유자에 대한 토큰 세트를 가져옵니다.
        let tokens_for_owner_set = self.tokens_per_owner.get(&account_id);

        //if there is some set of tokens, we'll return the length as a U128
        //토큰의 일부 세트가 있다면 U128로 길이를 반환합니다.
        if let Some(tokens_for_owner_set) = tokens_for_owner_set {
            U128(tokens_for_owner_set.len() as u128)
        } else {
            //if there isn't a set of tokens for the passed in account ID, we'll return 0
            //전달된 계정 ID에 대한 토큰 세트가 없다면 0을 반환합니다.
            U128(0)
        }
    }

    //Query for all the tokens for an owner
    //소유자의 모든 토큰을 쿼리합니다.
    /**
     * near view YOUR_CONTRACT_ID nft_tokens_for_owner '{"account_id": "TARGET_ACCOUNT_ID", "from_index": OPTIONAL_FROM_INDEX, "limit": OPTIONAL_LIMIT}'
        YOUR_CONTRACT_ID: 해당 NFT 컨트랙트의 계정 ID로 교체해야 합니다.
        TARGET_ACCOUNT_ID: 토큰 소유자의 계정 ID로 교체해야 합니다.
        OPTIONAL_FROM_INDEX: 시작 인덱스를 지정하려면, 예를 들어 "0", "1", "2" 등의 문자열로 교체하거나 필요하지 않은 경우 이 부분을 생략할 수 있습니다.
        OPTIONAL_LIMIT: 반환되는 토큰 수의 제한을 지정하려면, 예를 들어 
        Alice의 계정에서 처음 10개의 토큰을 조회하려면:
        near view YOUR_CONTRACT_ID nft_tokens_for_owner '{"account_id": "alice.near", "from_index": "0", "limit": "10"}'

     */
    pub fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonToken> {
        //get the set of tokens for the passed in owner
        //전달된 소유자에 대한 토큰 세트를 가져옵니다.
        let tokens_for_owner_set = self.tokens_per_owner.get(&account_id);
        //if there is some set of tokens, we'll set the tokens variable equal to that set
        //토큰의 일부 세트가 있다면 tokens 변수를 해당 세트와 동일하게 설정합니다.
        let tokens = if let Some(tokens_for_owner_set) = tokens_for_owner_set {
            tokens_for_owner_set
        } else {
            //if there is no set of tokens, we'll simply return an empty vector. 
            //토큰 세트가 없다면 그냥 빈 벡터를 반환합니다.
            return vec![];
        };

        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        //페이징을 시작할 위치 - from_index가 있으면 그것을 사용하고, 그렇지 않으면 0 인덱스에서 시작합니다.
        let start = u128::from(from_index.unwrap_or(U128(0)));

        //iterate through the keys vector
        //키 벡터를 순회합니다.
        tokens.iter()
            //skip to the index we specified in the start variable
            //start 변수에 지정한 인덱스로 건너뜁니다.
            .skip(start as usize) 
            //take the first "limit" elements in the vector. If we didn't specify a limit, use 50
            //벡터에서 첫 번째 "limit" 요소를 가져옵니다. limit을 지정하지 않았다면 50을 사용합니다.
            .take(limit.unwrap_or(50) as usize) 
            //we'll map the token IDs which are strings into Json Tokens
            //문자열인 토큰 ID를 Json 토큰으로 매핑합니다.
            .map(|token_id| self.nft_token(token_id.clone()).unwrap())
            //since we turned the keys into an iterator, we need to turn it back into a vector to return
            //키를 반복자로 변환했으므로 반환하기 위해 벡터로 다시 변환해야 합니다.
            .collect()
    }
}
