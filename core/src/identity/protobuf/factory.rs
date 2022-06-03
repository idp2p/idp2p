use chrono::Utc;
use cid::Cid;
use prost::Message;

use crate::{
    identity::{error::IdentityError, CreateIdentityInput, IdEvent},
    idp2p_proto::{IdentityInception, Microledger},
    multi::id::{Idp2pCid, Idp2pCodec},
};

pub fn new(input: CreateIdentityInput) -> Result<crate::identity::Identity, IdentityError> {
    let mut inception = IdentityInception {
        version: 1,
        timestamp: Utc::now().timestamp(),
        next_key_digest: input.next_key_digest.to_bytes(),
        recovery_key_digest: input.recovery_key_digest.to_bytes(),
        events: vec![],
    };
    for id_event in input.events {
        match id_event {
            IdEvent::RevokeAssertionKey(_)
            | IdEvent::RevokeAuthenticationKey(_)
            | IdEvent::RevokeAgreementKey(_) => {
                return Err(IdentityError::Other);
            }
            _ => inception.events.push(id_event.into()),
        }
    }

    let inception_bytes = inception.encode_to_vec();
    let cid = Cid::new_cid(Idp2pCodec::Protobuf, &inception_bytes);
    let microledger = Microledger {
        inception: inception_bytes,
        event_logs: vec![],
    };
    let did = crate::identity::Identity {
        id: cid.into(),
        microledger: microledger.encode_to_vec(),
    };
    Ok(did)
}
