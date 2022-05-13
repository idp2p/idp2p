use crate::{
    identity::input::IdEvents,
    idp2p_proto::{self, identity_event::EventType, EventLog, EventLogPayload},
    IdentityError,
};
use idp2p_common::{
    agreement_key::Idp2pAgreementKey,
    anyhow::{bail, Result},
    cid::multihash::Multihash,
    key::Idp2pKey,
    key_digest::Idp2pKeyDigest,
    Idp2pHasher,
};
use prost::Message;

pub trait EventLogResolver {
    fn try_resolve(&self, event_id: &[u8]) -> Result<EventLogPayload>;
}

impl EventLogResolver for EventLog {
    fn try_resolve(&self, event_id: &[u8]) -> Result<EventLogPayload> {
        let multi_id = Multihash::from_bytes(&self.event_id)?;
        if !multi_id.is_hash_of(&self.payload)? {
            bail!(IdentityError::InvalidId)
        }
        let payload = EventLogPayload::decode(&*self.payload)?;
        if payload.previous != event_id {
            bail!(IdentityError::InvalidPrevious)
        }
        Ok(payload)
    }
}

impl Into<idp2p_proto::Idp2pKeyDigest> for Idp2pKeyDigest {
    fn into(self) -> idp2p_proto::Idp2pKeyDigest {
        match self {
            Idp2pKeyDigest::Idp2pEd25519 { mh } => idp2p_proto::Idp2pKeyDigest {
                key_type: 0xed,
                digest: mh.to_bytes(),
            },
        }
    }
}

impl TryInto<Idp2pKeyDigest> for idp2p_proto::Idp2pKeyDigest {
    type Error = idp2p_common::anyhow::Error;

    fn try_into(self) -> Result<Idp2pKeyDigest, Self::Error> {
        Idp2pKeyDigest::try_from(self.key_type as u64, &self.digest)
    }
}

impl Into<idp2p_proto::Idp2pKey> for Idp2pKey {
    fn into(self) -> idp2p_proto::Idp2pKey {
        match self {
            Idp2pKey::Idp2pEd25519 { public } => idp2p_proto::Idp2pKey {
                key_type: 0xed,
                public: public.to_bytes().to_vec(),
            },
        }
    }
}

impl TryInto<Idp2pKey> for idp2p_proto::Idp2pKey {
    type Error = idp2p_common::anyhow::Error;

    fn try_into(self) -> Result<Idp2pKey, Self::Error> {
        Idp2pKey::try_from(self.key_type as u64, &self.public)
    }
}

impl Into<idp2p_proto::Idp2pAgreementKey> for Idp2pAgreementKey {
    fn into(self) -> idp2p_proto::Idp2pAgreementKey {
        match self {
            Idp2pAgreementKey::Idp2pX25519 { public } => idp2p_proto::Idp2pAgreementKey {
                key_type: 0xed,
                public: public.to_bytes().to_vec(),
            },
        }
    }
}

impl TryInto<Idp2pAgreementKey> for idp2p_proto::Idp2pAgreementKey {
    type Error = idp2p_common::anyhow::Error;

    fn try_into(self) -> Result<Idp2pAgreementKey, Self::Error> {
        Idp2pAgreementKey::try_from(self.key_type as u64, &self.public)
    }
}

impl Into<Vec<idp2p_proto::IdentityEvent>> for IdEvents {
    fn into(self) -> Vec<idp2p_proto::IdentityEvent> {
        let mut events: Vec<idp2p_proto::IdentityEvent> = vec![];
        if let Some(authentication_key) = self.authentication_key {
            events.push(idp2p_proto::IdentityEvent {
                event_type: Some(EventType::CreateAuthenticationKey(
                    authentication_key.into(),
                )),
            });
        }
        if let Some(agreement_key) = self.agreement_key {
            events.push(idp2p_proto::IdentityEvent {
                event_type: Some(EventType::CreateAgreementKey(agreement_key.into())),
            });
        }
        if let Some(assertion_key) = self.assertion_key {
            events.push(idp2p_proto::IdentityEvent {
                event_type: Some(EventType::CreateAssertionKey(assertion_key.into())),
            });
        }

        for (k, v) in self.proofs {
            events.push(idp2p_proto::IdentityEvent {
                event_type: Some(EventType::SetProof(idp2p_proto::Idp2pProof {
                    key: k,
                    value: v,
                })),
            });
        }
        events
    }
}
