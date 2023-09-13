//사용자 간에 NFT를 전송할 수 있는 핵심 로직입니다.
use crate::*;
use near_sdk::{ext_contract, Gas, PromiseResult};

const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(10_000_000_000_000);
const GAS_FOR_NFT_ON_TRANSFER: Gas = Gas(25_000_000_000_000);

pub trait NonFungibleTokenCore {
    //transfers an NFT to a receiver ID
    // NFT를 수신자 ID에게 전송합니다.
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        //we introduce an approval ID so that people with that approval ID can transfer the token
        // 전송 가능한 승인 ID를 도입합니다.
        approval_id: Option<u64>,
        memo: Option<String>,
    );

    /// Transfers an NFT to a receiver and calls the
    ///  function `nft_on_transfer` on their contract.\
    /// NFT를 수신자에게 전송하고 
    /// 수신자의 계약에 `nft_on_transfer` 함수를 호출합니다.
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        //we introduce an approval ID so that people with that approval ID can transfer the token
        // 전송 가능한 승인 ID를 도입합니다.
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool>;

    //get information about the NFT token passed in
    // 전달된 NFT 토큰에 대한 정보를 가져옵니다.
    fn nft_token(&self, token_id: TokenId) -> Option<JsonToken>;
}

// 계약 간 호출을 통해 수신자 계약에 저장된 메소드
#[ext_contract(ext_non_fungible_token_receiver)]
trait NonFungibleTokenReceiver {
    //Method stored on the receiver contract that is called via cross contract call when nft_transfer_call is called
    /// Returns `true` if the token should be returned back to the sender.
    /// nft_transfer_call이 호출될 때 교차 계약 호출을 통해 호출됩니다.
    /// 토큰을 발신자에게 반환해야하는 경우 `true`를 반환합니다.
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> Promise;
}

#[ext_contract(ext_self)]
/*
    resolves the promise of the cross contract call to the receiver contract
    this is stored on THIS contract and is meant to analyze what happened in the cross contract call when nft_on_transfer was called
    as part of the nft_transfer_call method
    수신자 계약에 대한 교차 계약 통화 promise 을 해결합니다
    이는 본 계약서에 저장되어 있으며, nft_transfer_call 메서드의 일부로 nft_on_transfer가 호출되었을 때 교차 계약 호출에서 무슨 일이 있었는지 분석하기 위한 것입니다
*/ 
trait NonFungibleTokenResolver {
    // nft_transfer_call 메서드에서 nft_on_transfer를 호출할 때 교차 계약 호출을 해결합니다.
    // 토큰이 receiver_id에게 성공적으로 전송되었는지 여부를 반환합니다. 
    fn nft_resolve_transfer(
        &mut self,
        //we introduce an authorized ID for logging the transfer event
        // 전송 이벤트를 기록하기 위해 승인 ID를 도입합니다.
        authorized_id: Option<String>,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        //we introduce the approval map so we can keep track of what the approvals were before the transfer
        // 전송 전 승인 상태를 추적하기 위해 승인 맵을 도입합니다.
        approved_account_ids: HashMap<AccountId, u64>,
        //we introduce a memo for logging the transfer event
        // 전송 이벤트를 기록하기 위해 메모를 도입합니다.
        memo: Option<String>,
    ) -> bool;
}

#[near_bindgen]
impl NonFungibleTokenCore for Contract {

    //implementation of the nft_transfer method. This transfers the NFT from the current owner to the receiver. 
    // nft_transfer 메소드의 구현입니다. 이는 현재 소유자로부터 수신자에게 NFT를 전송합니다.
    #[payable]
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        //we introduce an approval ID so that people with that approval ID can transfer the token
        // 전송 가능한 승인 ID를 도입합니다.
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        //assert that the user attached exactly 1 yoctoNEAR. This is for security and so that the user will be redirected to the NEAR wallet. 
        // 보안을 위해 사용자가 정확히 1 yoctoNEAR를 첨부했는지 확인합니다.
        assert_one_yocto();
        //get the sender to transfer the token from the sender to the receiver
        // 발신자를 가져와 발신자로부터 수신자에게 토큰을 전송합니다
        let sender_id = env::predecessor_account_id();

        //call the internal transfer method and get back the previous token so we can refund the approved account IDs
        // 내부 전송 메서드를 호출하고 이전 토큰을 반환받아 승인된 계정 ID를 환불합니다.
        let previous_token = self.internal_transfer(
            &sender_id,
            &receiver_id,
            &token_id,
            approval_id,
            memo,
        );

        //we refund the owner for releasing the storage used up by the approved account IDs
        // 승인된 계정 ID에 사용된 저장소를 해제함으로써 소유자에게 환불을 합니다.
        refund_approved_account_ids(
            previous_token.owner_id.clone(),
            &previous_token.approved_account_ids,
        );
    }

    //implementation of the transfer call method. This will transfer the NFT and call a method on the receiver_id contract
    // NFT를 전송하고 receiver_id 계약에 메서드를 호출하는 transfer call 메서드의 구현입니다.
    #[payable]
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        //we introduce an approval ID so that people with that approval ID can transfer the token
        // 전송 가능한 승인 ID를 도입합니다.
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        //assert that the user attached exactly 1 yocto for security reasons. 
        // 보안을 위해 사용자가 정확히 1 yoctoNEAR를 첨부했는지 확인합니다.
        assert_one_yocto();

        //get the sender ID 
        // 발신자 ID를 가져옵니다.
        let sender_id = env::predecessor_account_id();

        //transfer the token and get the previous token object
        // 토큰을 전송하고 이전 토큰 객체를 가져옵니다.
        let previous_token = self.internal_transfer(
            &sender_id,
            &receiver_id,
            &token_id,
            approval_id,
            memo.clone(),
        );

        //default the authorized_id to none
        // 기본적으로 authorized_id를 none으로 설정합니다.
        let mut authorized_id = None; 
        //if the sender isn't the owner of the token, we set the authorized ID equal to the sender.
        // 발신자가 토큰의 소유자가 아닌 경우 authorized_id를 발신자로 설정합니다   
        if sender_id != previous_token.owner_id {
            authorized_id = Some(sender_id.to_string());
        }

        // Initiating receiver's call and the callback
        // Defaulting GAS weight to 1, no attached deposit, and static GAS equal to the GAS for nft on transfer.
        // 수신자의 통화 및 콜백 시작
        // GAS weight를 1로 기본 설정하고, 부착된 보증금은 없으며, nft 전송 시 GAS와 동일한 정적 GAS.
        ext_non_fungible_token_receiver::ext(receiver_id.clone())
            .with_static_gas(GAS_FOR_NFT_ON_TRANSFER)
            .nft_on_transfer(
                sender_id, 
                previous_token.owner_id.clone(), 
                token_id.clone(), 
                msg
            )
        // We then resolve the promise and call nft_resolve_transfer on our own contract
        // 그런 다음 프라미스를 확인하고 우리의 계약에서 nft_resolve_transfer를 호출합니다.
        .then(
            // Defaulting GAS weight to 1, no attached deposit, and static GAS equal to the GAS for resolve transfer
            Self::ext(env::current_account_id())
                .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                .nft_resolve_transfer(
                    authorized_id, // we introduce an authorized ID so that we can log the transfer
                    previous_token.owner_id,
                    receiver_id,
                    token_id,
                    previous_token.approved_account_ids,
                    memo, // we introduce a memo for logging in the events standard
                )
        ).into()
    }

    //get the information for a specific token ID
    // 특정 토큰 ID에 대한 정보를 가져옵니다.
    fn nft_token(&self, token_id: TokenId) -> Option<JsonToken> {
        //if there is some token ID in the tokens_by_id collection
        // 토큰 ID가 tokens_by_id 컬렉션에 있다면
        if let Some(token) = self.tokens_by_id.get(&token_id) {
            //we'll get the metadata for that token
            // 해당 토큰의 메타데이터를 가져옵니다.
            let metadata = self.token_metadata_by_id.get(&token_id).unwrap();
            //we return the JsonToken (wrapped by Some since we return an option)
            // JsonToken을 반환합니다(Option으로 감싸서 반환합니다).
            Some(JsonToken {
                token_id,
                owner_id: token.owner_id,
                metadata,
                approved_account_ids: token.approved_account_ids,
                royalty: token.royalty,
            })
        } else { //if there wasn't a token ID in the tokens_by_id collection, we return None
                // 토큰 ID가 tokens_by_id 컬렉션에 없으면 None을 반환합니다.
            None
        }
    }
}

#[near_bindgen]
impl NonFungibleTokenResolver for Contract {
    //resolves the cross contract call when calling nft_on_transfer in the nft_transfer_call method
    //returns true if the token was successfully transferred to the receiver_id
    // nft_transfer_call 메서드에서 nft_on_transfer를 호출할 때 교차 계약 호출을 해결합니다.
    // 토큰이 receiver_id에게 성공적으로 전송되었는지 여부를 반환합니다.
    #[private]
    fn nft_resolve_transfer(
        &mut self,
        //we introduce an authorized ID for logging the transfer event
        // 전송 이벤트를 기록하기 위해 승인 ID를 도입합니다
        authorized_id: Option<String>,
        owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        //we introduce the approval map so we can keep track of what the approvals were before the transfer
        // 전송 전 승인 상태를 추적하기 위해 승인 맵을 도입합니다.
        approved_account_ids: HashMap<AccountId, u64>,
        //we introduce a memo for logging the transfer event
        // 전송 이벤트를 기록하기 위해 메모를 도입합니다.
        memo: Option<String>,
    ) -> bool {
        // Whether receiver wants to return token back to the sender, based on `nft_on_transfer`
        // call result.
        // receiver가 `nft_on_transfer` 호출 결과에 따라 토큰을 발신자에게 반환하고자 하는지 여부입니다.
        if let PromiseResult::Successful(value) = env::promise_result(0) {
            //As per the standard, the nft_on_transfer should return whether we should return the token to it's owner or not
            // 표준에 따르면, nft_on_transfer는 우리가 토큰을 원래 소유자에게 반환해야하는지 여부를 반환해야 합니다.
            if let Ok(return_token) = near_sdk::serde_json::from_slice::<bool>(&value) {
                //if we need don't need to return the token, we simply return true meaning everything went fine
                // 토큰을 반환할 필요가 없으면 true를 반환하여 모든 것이 잘 진행되었음을 나타냅니다.
                if !return_token {
                    /* 
                        since we've already transferred the token and nft_on_transfer returned false, we don't have to 
                        revert the original transfer and thus we can just return true since nothing went wrong.
                    // 이미 토큰을 전송하고 nft_on_transfer가 false를 반환했으므로 원래 전송을 되돌릴 필요가 없습니다.
                    // 그러므로 우리는 그냥 true를 반환하면서 문제가 없었음을 나타냅니다.
                    */
                    //we refund the owner for releasing the storage used up by the approved account IDs
                    //승인된 계정 ID에 의해 사용된 스토리지를 출고할 경우 소유자에게 환불해 드립니다
                    refund_approved_account_ids(owner_id, &approved_account_ids);
                    return true;
                }
            }
        }

        //get the token object if there is some token object
        // 토큰 객체를 가져옵니다.
        let mut token = if let Some(token) = self.tokens_by_id.get(&token_id) {
            if token.owner_id != receiver_id {
                //we refund the owner for releasing the storage used up by the approved account IDs
                //승인된 계정 ID에 의해 사용된 스토리지를 출고할 경우 소유자에게 환불해 드립니다
                refund_approved_account_ids(owner_id, &approved_account_ids);
                // The token is not owner by the receiver anymore. Can't return it.
                // 토큰이 더 이상 receiver에게 소유되어 있지 않습니다. 반환할 수 없습니다.
                return true;
            }
            token
        //if there isn't a token object, it was burned and so we return true
        //토큰 개체가 없으면 태웠기 때문에 참으로 돌아갑니다
        } else {
            //we refund the owner for releasing the storage used up by the approved account IDs
            //승인된 계정 ID에 의해 사용된 스토리지를 출고할 경우 소유자에게 환불해 드립니다
            refund_approved_account_ids(owner_id, &approved_account_ids);
            return true;
        };

        //we remove the token from the receiver
        // receiver에서 토큰을 제거합니다. 
        self.internal_remove_token_from_owner(&receiver_id.clone(), &token_id);
        //we add the token to the original owner
        // 원래 소유자에게 토큰을 추가합니다.
        self.internal_add_token_to_owner(&owner_id, &token_id);

        //we change the token struct's owner to be the original owner 
        // 토큰 구조의 소유자를 원래 소유자로 변경합니다.
        token.owner_id = owner_id.clone();

        //we refund the receiver any approved account IDs that they may have set on the token
        // receiver에게 토큰에 설정한 승인된 계정 ID를 환불합니다.
        refund_approved_account_ids(receiver_id.clone(), &token.approved_account_ids);
        //reset the approved account IDs to what they were before the transfer
        // 전송 전 승인 상태로 다시 설정합니다.
        token.approved_account_ids = approved_account_ids;

        //we inset the token back into the tokens_by_id collection
        // tokens_by_id 컬렉션에 토큰을 다시 삽입합니다.
        self.tokens_by_id.insert(&token_id, &token);

        /*
            We need to log that the NFT was reverted back to the original owner.
            The old_owner_id will be the receiver and the new_owner_id will be the
            original owner of the token since we're reverting the transfer.
            NFT가 원래 소유자에게 반환되었음을 기록해야 합니다.
            이전을 되돌리는 중이므로 old_owner_id가 수신자가 되고 new_owner_id가 토큰의 원래 소유자가 됩니다.
        */
        let nft_transfer_log: EventLog = EventLog {
            // Standard name ("nep171").
            standard: NFT_STANDARD_NAME.to_string(),
            // Version of the standard ("nft-1.0.0").
            version: NFT_METADATA_SPEC.to_string(),
            // The data related with the event stored in a vector.
            event: EventLogVariant::NftTransfer(vec![NftTransferLog {
                // The optional authorized account ID to transfer the token on behalf of the old owner.
                authorized_id,
                // The old owner's account ID.
                old_owner_id: receiver_id.to_string(),
                // The account ID of the new owner of the token.
                new_owner_id: owner_id.to_string(),
                // A vector containing the token IDs as strings.
                token_ids: vec![token_id.to_string()],
                // An optional memo to include.
                memo,
            }]),
        };

        //we perform the actual logging
        env::log_str(&nft_transfer_log.to_string());

        //return false
        false
    }
}