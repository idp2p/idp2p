pub struct Block {
    id: Vec<u8>,
    prev_hash: Vec<u8>,
    state_hash: Vec<u8>,
    tx_hash: Vec<u8>,
}

pub struct VerifiableCredential {
    // Digest of inception
    id: Vec<u8>,
    inception: Vec<u8>,
    mutations: Vec<Mutation>,
}

pub struct Inception {
    next_issuer_pk: Vec<u8>, // winternitz
    next_holder_pk: Vec<u8>, // ed25519
    assertions: Vec<Vec<u8>>,    // sha256
}

pub struct Mutation{
    id: Vec<u8>,
    payload: Vec<u8>,
    signature: Vec<u8>
}

// AddProof, Change Owner
pub struct MutationPayload {
    prev_hash: Vec<u8>,
    signer_pk: Vec<u8>,
    next_pk_hash: Vec<u8>,
    assertions: Vec<Vec<u8>>,
}


pub struct VerifiableCredentialState {
    pub id: Vec<u8>,
    pub last_mutation_id: Vec<u8>,
    pub next_issuer_pk_hash: Vec<u8>,
    pub next_holder_pk_hash: Vec<u8>,
    pub assertions: Vec<Vec<u8>>,
}

impl VerifiableCredential{
    pub fn verify(&self) -> VerifiableCredentialState{
        let id = Idp2pId::from_bytes(&self.id)?;
        // Check cid is produced with inception
        id.ensure(&self.inception)?;
        let mut state = VerifiableCredentialState {
            id: id.to_bytes(),
            last_mutation_id: self.inception,
            next_issuer_pk_hash: vec![],
            next_holder_pk_hash: vec![],
            assertions: vec![]
        };
        for mutation in self.mutations {
            let log_id = Idp2pId::from_bytes(&log.id)?;
            log_id.ensure(&log.payload)?;
            let payload = log.try_resolve_payload()?;
            // Previous event_id should match with last_event_id of state.
            if payload.previous != state.last_event_id {
                return Err(Idp2pError::InvalidPreviousEventLog);
            }
            let change = payload
                .change
                .ok_or(Idp2pError::RequiredField("change".to_string()))?;
            match change {
                ProtoMutationKind::Recover(key_digest) => {
                    let signer = state.next_rec_key(&payload.signer_key)?;
                    signer.verify(&log.payload, &log.proof)?;
                    state.recovery_key_digest = key_digest;
                }
                ProtoMutationKind::Events(events) => {
                    let signer = state.next_signer_key(&payload.signer_key)?;
                    signer.verify(&log.payload, &log.proof)?;
                    for event in events.events {
                        let event = event
                            .event_type
                            .ok_or(Idp2pError::RequiredField("event_type".to_string()))?;
                        state.handle_event(payload.timestamp, event)?;
                    }
                }
            }
            state.next_key_digest = payload.next_key_digest;
            state.last_event_id = log.id;
        }
    }
}

// issuer: revoke, change owner and add important proofs etc.
// holder: add proofs
// holder should keep all mutation metadata and issuer change content
// a proof about an event metadata
// how to verify ? holder should present all proofs metadata in wallet
