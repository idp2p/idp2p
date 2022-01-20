use crate::file_store::FileStore;
use crate::id_behaviour::IdentityGossipBehaviour;
use crate::id_message::{IdentityMessage, IdentityMessageType};
use idp2p_core::did::Identity;
use libp2p::gossipsub::IdentTopic;

#[derive(PartialEq, Debug, Clone)]
pub enum IdentityCommand {
    Post { did: Identity },
    Get { id: String },
    Jwm { id: String, message: String },
}

impl IdentityCommand {
    pub fn handle(&self, behaviour: &mut IdentityGossipBehaviour) {
        match self {
            IdentityCommand::Post { did } => {
                let id = did.id.clone();
                let gossipsub_topic = IdentTopic::new(did.id.clone());
                behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();
                if behaviour.identities.contains_key(&id) {
                    let post = IdentityMessageType::Post {
                        digest: did.get_digest(),
                        identity: did.clone(),
                    };
                    behaviour.publish(id.clone(), IdentityMessage::new(post));
                } else {
                    behaviour.identities.insert(id.clone(), did.get_digest());
                    FileStore.put("identities", &did.id, did);
                }
            }
            IdentityCommand::Get { id } => {
                let gossipsub_topic = IdentTopic::new(id.clone());
                behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();
                behaviour.publish(id.clone(), IdentityMessage::new(IdentityMessageType::Get));
            }
            IdentityCommand::Jwm { id, message } => {
                let gossipsub_topic = IdentTopic::new(id.clone());
                behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();
                let id_mes = IdentityMessage::new(IdentityMessageType::Jwm {
                    message: message.to_owned(),
                });
                behaviour.publish(id.clone(), id_mes);
            }
        }
    }
}
