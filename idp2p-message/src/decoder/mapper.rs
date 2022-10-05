use idp2p_common::multi::id::Idp2pId;
use prost::Message;

use crate::{
    error::Idp2pError,
    id_message::{IdMessage, IdMessageBody},
    identity::IdEvent,
    idp2p_proto::{
        self, identity_event::EventType, EventLog, EventLogPayload, IdentityEvent, Idp2pMultiKey,
        Idp2pProof,
    },
};

impl Into<IdMessage> for idp2p_proto::IdMessage {
    fn into(self) -> IdMessage {
        IdMessage {
            id: self.id,
            from: self.from,
            to: self.to,
            signer_kid: self.signer_kid,
            proof: self.proof,
            created_at: self.created_at,
            body: self.body.unwrap().into(),
            reply_to: None,
        }
    }
}

impl Into<IdMessageBody> for idp2p_proto::id_message::Body {
    fn into(self) -> IdMessageBody {
        match self{
            idp2p_proto::id_message::Body::Text(msg) => IdMessageBody::Text(msg),
        }
    }
}

impl Into<idp2p_proto::id_message::Body> for IdMessageBody {
    fn into(self) -> idp2p_proto::id_message::Body {
        match self{
            IdMessageBody::Text(msg) => idp2p_proto::id_message::Body::Text(msg),
        }
    }
}