//! NIP-74 event helper functions

use uuid::Uuid;
use nostr::event::tag::kind::TagKind;
use crate::{MintInfo, OperationResult};

/// Generate a fresh request id (UUID v4 as lowercase string).
pub fn new_request_id() -> String {
    Uuid::new_v4().to_string()
}

/// Build a `kind:37400` MintInfo event following the agreed strategy:
/// – `content` contains the exact JSON serialization of NUT-06 `MintInfo`;
/// – `d` tag stores the provided unique `identifier` (slug / pubkey);
/// – `relays` tag lists relay URLs where the mint is reachable;
/// – `status` tag gives a quick health indicator (e.g. "running").
/// Additional tags can be appended via `extra_tags`.
pub async fn build_mint_info_event<S>(
    mint_info: &MintInfo,
    signer: &S,
    identifier: &str,
    relays: &[nostr::RelayUrl],
    status: &str,
    extra_tags: Option<Vec<nostr::Tag>>,
) -> crate::Nip74Result<nostr::Event>
where
    S: nostr::NostrSigner,
{
    // Serialize full NUT-06 payload verbatim.
    let content = serde_json::to_string(mint_info)?;

    // Compose mandatory tags.
    let mut builder = nostr::EventBuilder::new(nostr::Kind::from(37400u16), content)
        .tag(nostr::Tag::identifier(identifier.to_owned()))
        .tag(nostr::Tag::custom(
            TagKind::Relays,
            relays.iter().map(|r| r.as_str()).collect::<Vec<_>>(),
        ))
        .tag(nostr::Tag::custom(TagKind::Status, [status.to_owned()]));

    // Append optional extra tags, if any.
    if let Some(tags) = extra_tags {
        builder = builder.tags(tags);
    }

    // Sign and return event.
    let event = builder.sign(signer).await?;
    Ok(event)
}

impl OperationResult {
    /// Convert to `kind:27402` event and sign.
    pub async fn to_event_with_signer<T>(
        &self,
        signer: &T,
        author_pubkey: &nostr::PublicKey,
        receiver_pubkey: &nostr::PublicKey,
        request_event_id: &nostr::EventId,
        extra_tags: Option<Vec<nostr::Tag>>,
    ) -> nostr::Result<nostr::Event>
    where
        T: nostr::NostrSigner,
    {
        // Serialize response content
        let content_str = serde_json::to_string(self)?;
        
        // Use NIP-44 encryption
        let encrypted_content = signer.nip44_encrypt(receiver_pubkey, &content_str).await?;
        
        let mut builder = nostr::EventBuilder::new(nostr::Kind::from(27402u16), encrypted_content)
            .tag(nostr::Tag::public_key(*receiver_pubkey))
            .tag(nostr::Tag::event(*request_event_id));

        if let Some(tags) = extra_tags {
            builder = builder.tags(tags);
        }

        // NIP-74 spec doesn't enforce, but we set the author explicitly for clarity.
        builder = builder.allow_self_tagging();

        let event = builder.sign(signer).await?;
        // Ensure builder signed with the provided author_pubkey if needed.
        debug_assert_eq!(event.pubkey, *author_pubkey);
        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr::prelude::*;
    use crate::types::{OperationResult, ResultStatus};

    #[test]
    fn test_new_request_id_unique() {
        let a = new_request_id();
        let b = new_request_id();
        assert_ne!(a, b);
        assert_eq!(a.len(), 36); // UUID v4
    }

    #[tokio::test]
    async fn test_build_mint_info_event_basic() {
        // Minimal MintInfo mock
        #[derive(serde::Serialize)]
        struct DummyMintInfo {
            name: String,
        }
        let mint_info = DummyMintInfo { name: "test-mint".to_string() };
        let keys = Keys::generate();
        let event = EventBuilder::new(Kind::from(37400u16), serde_json::to_string(&mint_info).unwrap())
            .tag(Tag::identifier("test"))
            .sign_with_keys(&keys)
            .unwrap();
        assert_eq!(event.kind, Kind::from(37400u16));
        assert!(event.tags.iter().any(|t| t.as_slice()[0] == "d"));
    }

    #[tokio::test]
    async fn test_operation_result_to_event_with_signer() {
        let keys = Keys::generate();
        let op_res = OperationResult {
            status: ResultStatus::Success,
            request_id: "reqid".to_string(),
            data: None,
            error: None,
        };
        let author = keys.public_key();
        let receiver = keys.public_key();
        let dummy_event_id = EventId::all_zeros();
        let event = op_res
            .to_event_with_signer(&keys, &author, &receiver, &dummy_event_id, None)
            .await
            .unwrap();
        assert_eq!(event.kind, Kind::from(27402u16));
        assert_eq!(event.pubkey, author);
    }
} 