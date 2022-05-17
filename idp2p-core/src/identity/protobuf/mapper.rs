use crate::{
    identity::input::IdEvents,
    idp2p_proto::{self, identity_event::EventType, EventLog, EventLogPayload},
    IdentityError,
};
use idp2p_common::{
    anyhow::{bail, Result},
    digest::Idp2pDigest,
    digest::Idp2pKeyDigest,
    key::{Idp2pAgreementKey, Idp2pKey},
    ED25519_CODE, SHA256_CODE, X25519_CODE,
};
use prost::Message;

pub trait EventLogResolver {
    fn try_resolve_payload(&self, event_id: &Idp2pDigest) -> Result<EventLogPayload>;
}

impl EventLogResolver for EventLog {
    fn try_resolve_payload(&self, event_id: &Idp2pDigest) -> Result<EventLogPayload> {
        let digest = self
            .event_id
            .to_owned()
            .ok_or(IdentityError::InvalidProtobuf)?;
        let digest: Idp2pDigest = digest.try_into()?;
        if !digest.is_hash_of(&self.payload) {
            bail!(IdentityError::InvalidId)
        }
        let payload = EventLogPayload::decode(&*self.payload)?;
        let prev: Idp2pDigest = payload
            .clone()
            .previous
            .ok_or(IdentityError::InvalidProtobuf)?
            .try_into()?;
        if prev != *event_id {
            bail!(IdentityError::InvalidPrevious)
        }
        Ok(payload)
    }
}

impl Into<idp2p_proto::Idp2pDigest> for Idp2pDigest {
    fn into(self) -> idp2p_proto::Idp2pDigest {
        match self {
            Idp2pDigest::Sha256 { digest } => idp2p_proto::Idp2pDigest {
                code: SHA256_CODE as i32,
                digest: digest,
            },
        }
    }
}

impl TryInto<Idp2pDigest> for idp2p_proto::Idp2pDigest {
    type Error = idp2p_common::anyhow::Error;
    fn try_into(self) -> Result<Idp2pDigest, Self::Error> {
        Idp2pDigest::from(self.code as u64, &self.digest)
    }
}

impl TryInto<Idp2pKey> for idp2p_proto::Idp2pKey {
    type Error = idp2p_common::anyhow::Error;

    fn try_into(self) -> Result<Idp2pKey, Self::Error> {
        Idp2pKey::new(self.code as u64, &self.public)
    }
}

impl Into<idp2p_proto::Idp2pKey> for Idp2pKey {
    fn into(self) -> idp2p_proto::Idp2pKey {
        match self {
            Idp2pKey::Idp2pEd25519 { public } => idp2p_proto::Idp2pKey {
                code: ED25519_CODE as i32,
                public: public,
            },
        }
    }
}

impl Into<idp2p_proto::Idp2pKeyDigest> for Idp2pKeyDigest {
    fn into(self) -> idp2p_proto::Idp2pKeyDigest {
        match self {
            Idp2pKeyDigest::Idp2pEd25519 { digest } => idp2p_proto::Idp2pKeyDigest {
                code: ED25519_CODE as i32,
                digest: Some(digest.into()),
            },
        }
    }
}
impl TryInto<Idp2pKeyDigest> for idp2p_proto::Idp2pKeyDigest {
    type Error = idp2p_common::anyhow::Error;

    fn try_into(self) -> Result<Idp2pKeyDigest, Self::Error> {
        Ok(Idp2pKeyDigest::new(
            self.code as u64,
            self.digest
                .ok_or(IdentityError::InvalidProtobuf)?
                .try_into()?,
        )?)
    }
}

impl Into<idp2p_proto::Idp2pAgreementKey> for Idp2pAgreementKey {
    fn into(self) -> idp2p_proto::Idp2pAgreementKey {
        match self {
            Idp2pAgreementKey::Idp2pX25519 { public } => idp2p_proto::Idp2pAgreementKey {
                code: X25519_CODE as i32,
                public: public,
            },
        }
    }
}

/*impl TryInto<Idp2pAgreementKey> for idp2p_proto::Idp2pAgreementKey {
    type Error = idp2p_common::anyhow::Error;

    fn try_into(self) -> Result<Idp2pAgreementKey, Self::Error> {
        Idp2pAgreementKey::new(self.code as u64, &self.public)
    }
}*/

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
