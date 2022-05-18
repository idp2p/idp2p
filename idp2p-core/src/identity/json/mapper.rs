use crate::identity::input::IdEvents;

use super::{EventType, Idp2pProof};

impl Into<Vec<EventType>> for IdEvents {
    fn into(self) -> Vec<EventType> {
        let mut events: Vec<EventType> = vec![];
        if let Some(authentication_key) = self.authentication_key {
            events.push(EventType::CreateAuthenticationKey {
                key: authentication_key,
            });
        }
        if let Some(agreement_key) = self.agreement_key {
            events.push(EventType::CreateAgreementKey { key: agreement_key });
        }
        if let Some(assertion_key) = self.assertion_key {
            events.push(EventType::CreateAssertionKey { key: assertion_key });
        }

        for (k, v) in self.proofs {
            events.push(EventType::SetProof {
                proof: Idp2pProof { key: k, value: v },
            });
        }
        events
    }
}
