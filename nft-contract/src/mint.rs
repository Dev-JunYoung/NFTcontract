// 토큰 발행 논리가 포함되어 있습니다.
use crate::*;

#[near_bindgen]
impl Contract {
    /**
     * 
     * near call CONTRACT_ID nft_mint '{
        "token_id": "YOUR_TOKEN_ID",
        "metadata": {
            "title": "TOKEN_TITLE",
            "description": "TOKEN_DESCRIPTION",
            ...other_metadata_fields...
        },
        "receiver_id": "RECEIVER_ACCOUNT_ID",
        "perpetual_royalties": {
            "ACCOUNT_ID_1": ROYALTY_PERCENTAGE_1,
            "ACCOUNT_ID_2": ROYALTY_PERCENTAGE_2,
            ...
        }
    }' --accountId YOUR_ACCOUNT_ID --amount SOME_AMOUNT
     */
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        //metadata: TokenMetadata,
        metadata: TokenMetadata,
        receiver_id: AccountId,
        // 영구 로열티를 위한 선택적 매개변수 추가
        perpetual_royalties: Option<HashMap<AccountId, u32>>,
    ) {
        // 계약에서 사용하는 초기 저장소를 측정합니다.
        let initial_storage_usage = env::storage_usage();

        // 토큰에 저장할 로열티 맵을 생성합니다.
        let mut royalty = HashMap::new();

        // 영구 로열티가 함수에 전달되면:
        if let Some(perpetual_royalties) = perpetual_royalties {
            // 영구 로열티의 길이가 7 미만인지 확인합니다. 그렇지 않으면 많은 사람들에게 GAS를 지급할 수 없습니다.
            assert!(perpetual_royalties.len() < 7, "Cannot add more than 6 perpetual royalty amounts");
            // 영구 로열티를 반복하고 로열티 맵에 계정과 금액을 삽입합니다.
            for (account, amount) in perpetual_royalties {
                royalty.insert(account, amount);
            }
        }

        // 소유자 ID를 포함하는 토큰 구조를 지정합니다.
        let token = Token {
            // 함수에 전달된 수신자 ID와 동일한 소유자 ID를 설정합니다.
            owner_id: receiver_id,
            // 승인된 계정 ID를 기본값(빈 맵)으로 설정합니다.
            approved_account_ids: Default::default(),
            // 다음 승인 ID를 0으로 설정합니다.
            next_approval_id: 0,
            // 토큰의 영구 로열티 맵 (소유자는 100% - 총 영구 로열티를 받게 됩니다.)
            royalty,
        };

        // 토큰 ID와 토큰 구조를 삽입하고 토큰이 이미 존재하지 않는지 확인합니다.
        assert!(
            self.tokens_by_id.insert(&token_id, &token).is_none(),
            "Token already exists"
        );

        // 토큰 ID와 메타데이터를 삽입합니다.
        self.token_metadata_by_id.insert(&token_id, &metadata);

        // 소유자에게 토큰을 추가하기 위한 내부 메서드를 호출합니다.
        self.internal_add_token_to_owner(&token.owner_id, &token_id);

        // 이벤트 표준에 따라 발행 로그를 구성합니다.
        let nft_mint_log: EventLog = EventLog {
            // 표준 이름 ("nep171").
            standard: NFT_STANDARD_NAME.to_string(),
            // 표준의 버전 ("nft-1.0.0").
            version: NFT_METADATA_SPEC.to_string(),
            // 이벤트와 관련된 데이터는 벡터에 저장됩니다.
            event: EventLogVariant::NftMint(vec![NftMintLog {
                // 토큰의 소유자.
                owner_id: token.owner_id.to_string(),
                // 발행된 토큰 ID의 벡터.
                token_ids: vec![token_id.to_string()],
                // 포함할 선택적 메모.
                memo: None,
            }]),
        };

        // 직렬화된 json을 로그에 기록합니다.
        env::log_str(&nft_mint_log.to_string());

        // 사용된 저장소 - 초기 저장소로 필요한 저장소를 계산합니다.
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        // 사용자가 너무 많이 첨부한 경우 초과 저장소를 환불합니다. 충분한 금액을 첨부하지 않아 필요한 경우에는 패닉이 발생합니다.
        refund_deposit(required_storage_in_bytes);
    }
}
