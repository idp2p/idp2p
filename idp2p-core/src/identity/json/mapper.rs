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
