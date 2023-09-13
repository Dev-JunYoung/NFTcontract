//대체 불가능한 토큰의 액세스 및 전송을 제어하는 ​​기능이 있습니다.
//이를 통해 사람들은 자신을 대신하여 NFT를 전송하도록 다른 계정을 승인할 수 있습니다.
//이 파일에는 표준의 승인 관리 확장을 준수하는 논리가 포함되어 있습니다. 다음은 메소드와 해당 기능에 대한 분석입니다.
use crate::*;
use near_sdk::{ext_contract};

pub trait NonFungibleTokenCore {
    //approve an account ID to transfer a token on your behalf
    //당신을 대신하여 토큰을 전송할 계정 ID 승인
    fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>);

    //check if the passed in account has access to approve the token ID
    //전달된 계정이 토큰 ID를 승인할 수 있는지 확인
	fn nft_is_approved(
        &self,
        token_id: TokenId,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool;

    //revoke a specific account from transferring the token on your behalf
    //당신을 대신하여 토큰을 전송하는 특정 계정의 승인 취소
    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId);

    //revoke all accounts from transferring the token on your behalf
    //당신을 대신하여 토큰을 전송하는 모든 계정의 승인 취소
    fn nft_revoke_all(&mut self, token_id: TokenId);
}

#[ext_contract(ext_non_fungible_approval_receiver)]
trait NonFungibleTokenApprovalsReceiver {
    //cross contract call to an external contract that is initiated during nft_approve
    //nft_approve 중에 시작된 외부 계약에 대한 교차 계약 호출
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    );
}

#[near_bindgen]
impl NonFungibleTokenCore for Contract {

    //allow a specific account ID to approve a token on your behalf
    //당신을 대신하여 토큰을 승인할 특정 계정 ID 허용
    /**
     * near call YOUR_CONTRACT_ID nft_approve 
     * '{"token_id": "YOUR_TOKEN_ID", "account_id": "ACCOUNT_TO_APPROVE", "msg": "OPTIONAL_MESSAGE"}' 
     * --accountId YOUR_ACCOUNT_ID --amount ATTACHED_AMOUNT
     * 
        YOUR_CONTRACT_ID는 스마트 컨트랙트가 배포된 계정의 ID입니다.
        YOUR_TOKEN_ID는 승인하려는 토큰의 ID입니다.
        ACCOUNT_TO_APPROVE는 승인하려는 계정의 ID입니다.
        OPTIONAL_MESSAGE는 선택적 메시지입니다 (필요한 경우에만).
        YOUR_ACCOUNT_ID는 함수를 호출하는 계정의 ID입니다. 이 계정은 토큰의 소유자여야 합니다.
        ATTACHED_AMOUNT는 첨부할 NEAR 토큰의 양입니다. 이는 특히 저장 비용을 위해 필요할 수 있습니다.
     */
    #[payable]
    fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>) {
        /*
            assert at least one yocto for security reasons - this will cause a redirect to the NEAR wallet.
            The user needs to attach enough to pay for storage on the contract
            보안상의 이유로 적어도 하나의 yocto를 확인 - 이것은 NEAR 지갑으로 리디렉션을 유발합니다.
            사용자는 계약에 저장료를 지불하기 위해 충분한 금액을 첨부해야 합니다.
        */
        assert_at_least_one_yocto();

        //get the token object from the token ID
         //토큰 ID에서 토큰 객체 가져오기
        let mut token = self.tokens_by_id.get(&token_id).expect("No token");

        //make sure that the person calling the function is the owner of the token
        //함수를 호출한 사람이 토큰의 소유자인지 확인
        assert_eq!(
            &env::predecessor_account_id(),
            &token.owner_id,
            "Predecessor must be the token owner."
        );

        //get the next approval ID if we need a new approval
        //새로운 승인이 필요하면 다음 승인 ID 가져오기
        let approval_id: u64 = token.next_approval_id;

        //check if the account has been approved already for this token
         //해당 계정이 이 토큰에 대해 이미 승인되었는지 확인
        let is_new_approval = token
            .approved_account_ids
            //insert returns none if the key was not present.  
            //키가 없으면 insert는 none을 반환합니다.
            .insert(account_id.clone(), approval_id)
            //if the key was not present, .is_none() will return true so it is a new approval.
            //키가 없으면, .is_none()은 true를 반환하므로 새로운 승인입니다.
            .is_none();

        //if it was a new approval, we need to calculate how much storage is being used to add the account.
        //새로운 승인이었으면 계정을 추가하여 사용되는 저장 공간을 계산합니다.
        let storage_used = if is_new_approval {
            bytes_for_approved_account_id(&account_id)
        //if it was not a new approval, we used no storage.
        //새로운 승인이 아니었다면 저장 공간을 사용하지 않았습니다.
        } else {
            0
        };

        //increment the token's next approval ID by 1
        //토큰의 다음 승인 ID를 1 증가
        token.next_approval_id += 1;
        //insert the token back into the tokens_by_id collection
        //tokens_by_id 컬렉션에 토큰 다시 삽입
        self.tokens_by_id.insert(&token_id, &token);

        //refund any excess storage attached by the user. If the user didn't attach enough, panic. 
        //사용자가 첨부한 불필요한 저장을 환불합니다. 사용자가 충분히 첨부하지 않았다면 panic. 
        refund_deposit(storage_used);

        //if some message was passed into the function, we initiate a cross contract call on the
        //account we're giving access to. 
        //함수에 메시지가 전달되면 접근 권한을 부여받은 계정에 대한 교차 계약 호출을 시작합니다. 
        if let Some(msg) = msg {
            // Defaulting GAS weight to 1, no attached deposit, and no static GAS to attach.
            //기본 GAS 무게를 1로 설정하고, 첨부된 입금 없음, 그리고 첨부할 정적 GAS 없음.
            ext_non_fungible_approval_receiver::ext(account_id)
                .nft_on_approve(
                    token_id, 
                    token.owner_id, 
                    approval_id, 
                    msg
                ).as_return();
        }
    }

     //전달된 계정이 토큰 ID를 승인할 수 있는지 확인
     // near call CONTRACT_ID nft_is_approved 
     // '{"token_id": "YOUR_TOKEN_ID", "approved_account_id": "YOUR_APPROVED_ACCOUNT_ID", "approval_id": OPTIONAL_APPROVAL_ID}' 
     // --accountId YOUR_ACCOUNT_ID
     /**
      * CONTRACT_ID: 스마트 컨트랙트의 NEAR 계정 ID입니다.
        YOUR_TOKEN_ID: 승인을 확인하려는 토큰의 ID입니다.
        YOUR_APPROVED_ACCOUNT_ID: 승인을 확인하려는 계정의 NEAR ID입니다.
        OPTIONAL_APPROVAL_ID: (선택 사항) 특정 승인 ID를 제공하려면 이 값을 설정하십시오. 그렇지 않으면 이 인자를 생략하십시오.
        YOUR_ACCOUNT_ID: 이 함수를 호출하는 NEAR 계정 ID입니다.
      */
    //near call nft.contract.near nft_is_approved '{"token_id": "token-123", "approved_account_id": "alice.near"}' --accountId bob.near
    //이 예에서는 token-123 토큰이 alice.near 계정에 의해 승인되었는지 확인합니다.
    // call 이지만 위에서 제공된 nft_is_approved 함수의 경우, 이 함수는 특정 계정이 토큰에 대한 승인을 확인하는 데 사용되므로 토큰을 첨부할 필요가 없습니다.
    //만약 함수가 [payable]로 표시되어 있거나 토큰을 첨부해야 하는 경우, --amount 플래그를 사용하여 원하는 금액을 첨부할 수 있습니다:
	fn nft_is_approved(
        &self,
        token_id: TokenId,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool {
        //토큰 ID에서 토큰 객체 가져오기
        let token = self.tokens_by_id.get(&token_id).expect("No token");

        //전달된 계정 ID에 대한 승인 번호 가져오기
		let approval = token.approved_account_ids.get(&approved_account_id);

        //계정 ID에 대한 승인 ID가 있으면
        if let Some(approval) = approval {
            //함수에 특정 approval_id가 전달되면
			if let Some(approval_id) = approval_id {
                //전달된 승인 ID가 계정의 실제 승인 ID와 일치하는지 반환
				approval_id == *approval
            //함수에 approval_id가 전달되지 않았으면 true 반환
			} else {
				true
			}
        //계정 ID에 대한 승인 ID가 없으면 false 반환
		} else {
			false
		}
    }

    //당신을 대신하여 토큰을 전송하는 특정 계정의 승인 취소
    // near call CONTRACT_ID nft_revoke '{"token_id": "YOUR_TOKEN_ID", "account_id": "ACCOUNT_TO_REVOKE"}'
    // --accountId YOUR_ACCOUNT_ID --amount 0.000000000000000000000001
    /**
     *  CONTRACT_ID는 NFT 스마트 계약의 계정 ID입니다.
        YOUR_TOKEN_ID는 승인을 취소하려는 토큰의 ID입니다.
        ACCOUNT_TO_REVOKE는 승인을 취소하려는 계정의 ID입니다.
        YOUR_ACCOUNT_ID는 토큰의 소유자 계정 ID입니다.
        --amount 0.000000000000000000000001는 1 yoctoNEAR를 첨부합니다.
     * 
     */
    #[payable]
    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId) {
        //보안상의 이유로 호출자가 정확히 1 yoctoNEAR를 첨부했는지 확인
        assert_one_yocto();
        //전달된 토큰 ID를 사용하여 토큰 객체 가져오기
        let mut token = self.tokens_by_id.get(&token_id).expect("No token");

        //함수 호출자 가져오기 및 토큰의 소유자인지 확인
        let predecessor_account_id = env::predecessor_account_id();
        assert_eq!(&predecessor_account_id, &token.owner_id);

        //계정 ID가 토큰의 승인에 있으면 제거하고 if 문 로직 실행
        if token
            .approved_account_ids
            .remove(&account_id)
            .is_some()
        {
            //approved_account_id를 제거함으로써 해제된 자금을 함수의 호출자에게 환불
            refund_approved_account_ids_iter(predecessor_account_id, [account_id].iter());

            //승인 목록에서 account_id를 제거하여 tokens_by_id 컬렉션에 토큰 다시 삽입
            self.tokens_by_id.insert(&token_id, &token);
        }
    }

    //당신을 대신하여 토큰을 전송하는 모든 계정의 승인 취소
    //near call CONTRACT_ID nft_revoke_all '{"token_id": "YOUR_TOKEN_ID"}' --accountId YOUR_ACCOUNT_ID --amount SOME_AMOUNT
    /**
     *  CONTRACT_ID는 NFT 스마트 계약의 계정 ID입니다.
        YOUR_TOKEN_ID는 승인을 취소하려는 토큰의 ID입니다.
        YOUR_ACCOUNT_ID는 토큰의 소유자 계정 ID입니다.
        SOME_AMOUNT는 첨부할 NEAR의 양입니다. 
     */
    #[payable]
    fn nft_revoke_all(&mut self, token_id: TokenId) {
        //보안을 위해 호출자가 정확히 1 yoctoNEAR를 첨부했는지 확인
        assert_one_yocto();

        //전달된 토큰 ID에서 토큰 객체 가져오기
        let mut token = self.tokens_by_id.get(&token_id).expect("No token");
        //호출자 가져오기 및 토큰의 소유자인지 확인
        let predecessor_account_id = env::predecessor_account_id();
        assert_eq!(&predecessor_account_id, &token.owner_id);

        //토큰의 승인된 계정 ID가 비어 있지 않으면 승인 취소
        if !token.approved_account_ids.is_empty() {
            //승인된 계정 ID를 함수 호출자에게 환불
            refund_approved_account_ids(predecessor_account_id, &token.approved_account_ids);
            //승인된 계정 ID 지우기
            token.approved_account_ids.clear();
            //승인된 계정 ID가 지워진 상태로 tokens_by_id 컬렉션에 토큰 다시 삽입
            self.tokens_by_id.insert(&token_id, &token);
        }
    }
}