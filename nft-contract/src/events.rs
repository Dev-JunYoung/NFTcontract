use std::fmt;

use near_sdk::serde::{Deserialize, Serialize};

/// Enum that represents the data type of the EventLog.
/// The enum can either be an NftMint or an NftTransfer.
/// 이벤트 로그의 데이터 유형을 나타내는 Enum입니다.
/// Enum은 NftMint 또는 NftTransfer 중 하나일 수 있습니다
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[serde(crate = "near_sdk::serde")]
#[non_exhaustive]
pub enum EventLogVariant {
    NftMint(Vec<NftMintLog>),
    NftTransfer(Vec<NftTransferLog>),
}

/// Interface to capture data about an event
///
/// Arguments:
/// * `standard`: name of standard e.g. nep171
/// * `version`: e.g. 1.0.0
/// * `event`: associate event data
/// 이벤트에 대한 데이터를 캡쳐하는 인터페이스입니다.
///
/// 인자:
/// * `standard`: 표준의 이름. 예: nep171
/// * `version`: 예: 1.0.0
/// * `event`: 연관된 이벤트 데이터
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct EventLog {
    pub standard: String,
    pub version: String,

    // `flatten` to not have "event": {<EventLogVariant>} in the JSON, just have the contents of {<EventLogVariant>}.
    #[serde(flatten)]
    pub event: EventLogVariant,
}

impl fmt::Display for EventLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "EVENT_JSON:{}",
            &serde_json::to_string(self).map_err(|_| fmt::Error)?
        ))
    }
}

/// An event log to capture token minting
///
/// Arguments
/// * `owner_id`: "account.near"
/// * `token_ids`: ["1", "abc"]
/// * `memo`: optional message
/// 토큰 발행을 캡쳐하는 이벤트 로그입니다.
///
/// 인자:
/// * `owner_id`: "account.near"
/// * `token_ids`: ["1", "abc"]
/// * `memo`: 선택적 메시지
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct NftMintLog {
    pub owner_id: String,
    pub token_ids: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

/// An event log to capture token transfer
///
/// Arguments
/// * `authorized_id`: approved account to transfer
/// * `old_owner_id`: "owner.near"
/// * `new_owner_id`: "receiver.near"
/// * `token_ids`: ["1", "12345abc"]
/// * `memo`: optional message
/// /// 토큰 전송을 캡쳐하는 이벤트 로그입니다.
///
/// 인자:
/// * `authorized_id`: 전송을 승인한 계정
/// * `old_owner_id`: "owner.near"
/// * `new_owner_id`: "receiver.near"
/// * `token_ids`: ["1", "12345abc"]
/// * `memo`: 선택적 메시지
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct NftTransferLog {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_id: Option<String>,

    pub old_owner_id: String,
    pub new_owner_id: String,
    pub token_ids: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nep_format_vector() {
        let expected = r#"EVENT_JSON:{"standard":"nep171","version":"1.0.0","event":"nft_mint","data":[{"owner_id":"foundation.near","token_ids":["aurora","proximitylabs"]},{"owner_id":"user1.near","token_ids":["meme"]}]}"#;
        let log = EventLog {
            standard: "nep171".to_string(),
            version: "1.0.0".to_string(),
            event: EventLogVariant::NftMint(vec![
                NftMintLog {
                    owner_id: "foundation.near".to_owned(),
                    token_ids: vec!["aurora".to_string(), "proximitylabs".to_string()],
                    memo: None,
                },
                NftMintLog {
                    owner_id: "user1.near".to_owned(),
                    token_ids: vec!["meme".to_string()],
                    memo: None,
                },
            ]),
        };
        assert_eq!(expected, log.to_string());
    }

    #[test]
    fn nep_format_mint() {
        let expected = r#"EVENT_JSON:{"standard":"nep171","version":"1.0.0","event":"nft_mint","data":[{"owner_id":"foundation.near","token_ids":["aurora","proximitylabs"]}]}"#;
        let log = EventLog {
            standard: "nep171".to_string(),
            version: "1.0.0".to_string(),
            event: EventLogVariant::NftMint(vec![NftMintLog {
                owner_id: "foundation.near".to_owned(),
                token_ids: vec!["aurora".to_string(), "proximitylabs".to_string()],
                memo: None,
            }]),
        };
        assert_eq!(expected, log.to_string());
    }

    #[test]
    fn nep_format_transfer_all_fields() {
        let expected = r#"EVENT_JSON:{"standard":"nep171","version":"1.0.0","event":"nft_transfer","data":[{"authorized_id":"market.near","old_owner_id":"user1.near","new_owner_id":"user2.near","token_ids":["token"],"memo":"Go Team!"}]}"#;
        let log = EventLog {
            standard: "nep171".to_string(),
            version: "1.0.0".to_string(),
            event: EventLogVariant::NftTransfer(vec![NftTransferLog {
                authorized_id: Some("market.near".to_string()),
                old_owner_id: "user1.near".to_string(),
                new_owner_id: "user2.near".to_string(),
                token_ids: vec!["token".to_string()],
                memo: Some("Go Team!".to_owned()),
            }]),
        };
        assert_eq!(expected, log.to_string());
    }
} 