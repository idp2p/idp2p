use crate::file_store::FileStore;
use crate::id_behaviour::IdentityGossipBehaviour;
use crate::id_message::{IdentityMessage, IdentityMessageType};
use idp2p_core::did::Identity;
use libp2p::gossipsub::IdentTopic;

#[derive(PartialEq, Debug, Clone)]
pub enum IdentityCommand {
    Post { did: Identity },
    Get { id: String },
}

impl IdentityCommand {
    pub fn handle(&self, behaviour: &mut IdentityGossipBehaviour) {
        match self {
            IdentityCommand::Post { did } => {
                // validate did
                // if valid and next change local store
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
        }
    }
}

