//토큰 및 메타데이터 구조를 정의합니다.
use crate::*;
pub type TokenId = String;

// defines the payout type we'll be returning as a part of the royalty standards.
// 로열티 표준의 일부로 반환될 지급 유형을 정의합니다.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

// NFT의 메타데이터 구조를 정의합니다.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NFTContractMetadata {
    // 필수, 예: "nft-1.0.0" 같은 버전
    pub spec: String,
    // 필수, 예: "Mosaics"
    pub name: String,
    // 필수, 예: "MOSAIC"
    pub symbol: String,
    // 데이터 URL
    pub icon: Option<String>,
    // 중앙화된 게이트웨이 URL, `reference` 또는 `media` URLs로 참조된 분산 저장소에 안정적인 액세스가 가능합니다.
    pub base_uri: Option<String>,
    // 추가 정보를 위한 JSON 파일 URL
    pub reference: Option<String>,
    // `reference` 필드의 JSON에 대한 Base64-encoded sha256 해시. `reference`가 포함되면 필요합니다.
    pub reference_hash: Option<Base64VecU8>,
}



// 토큰의 메타데이터 구조를 정의합니다.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    /// 토큰의 제목입니다.
    pub title: Option<String>,
    /// 토큰의 상세 설명입니다.
    pub description: Option<String>,
    /// 연결된 미디어의 URL입니다. 분산된, 내용 주소 지정 저장소를 선호합니다.
    pub media: Option<String>,
    /// `media` 필드에서 참조하는 내용의 Base64-encoded sha256 해시입니다. `media`가 포함되면 필요합니다.
    pub media_hash: Option<Base64VecU8>,
    /// 해당 토큰의 복사본 수입니다.
    pub copies: Option<u64>,
    /// 토큰 발행일입니다.
    pub issued_at: Option<String>,
    /// 토큰의 만료일입니다.
    pub expires_at: Option<String>,
    /// 토큰이 유효한 시작일입니다.
    pub starts_at: Option<String>,
    /// 토큰의 마지막 업데이트 일입니다.
    pub updated_at: Option<String>,
    /// NFT가 체인 상에 저장하려는 추가 정보입니다. 문자열화된 JSON일 수 있습니다.
    pub extra: Option<String>,
    /// 토큰에 관련된 참조 또는 문서의 URL입니다.
    pub reference: Option<String>,
    /// `reference` 필드에서 참조하는 내용의 Base64-encoded sha256 해시입니다.
    pub reference_hash: Option<Base64VecU8>,
}

// 토큰 구조를 정의합니다.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Token {
    // 토큰의 소유자
    pub owner_id: AccountId,
    pub approved_account_ids: HashMap<AccountId, u64>,
    pub next_approval_id: u64,
    // 토큰의 로열티 비율을 해시 맵에서 추적합니다.
    pub royalty: HashMap<AccountId, u32>,
}

// JsonToken은 뷰 호출에서 반환됩니다.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonToken {
    pub token_id: TokenId,
    pub owner_id: AccountId,
    pub metadata: TokenMetadata,
    pub approved_account_ids: HashMap<AccountId, u64>,
    pub royalty: HashMap<AccountId, u32>,
}

pub trait NonFungibleTokenMetadata {
    // 계약 메타데이터를 반환하기 위한 뷰 호출
    fn nft_metadata(&self) -> NFTContractMetadata;
}

#[near_bindgen]
impl NonFungibleTokenMetadata for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}
