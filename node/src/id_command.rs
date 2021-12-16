use core::did_doc::IdDocument;
use crate::id_behaviour::IdentityGossipBehaviour;
use libp2p::gossipsub::IdentTopic;

#[derive(PartialEq, Debug, Clone)]
pub enum IdentityCommand {
    Get {
        id: String
    },
    SetProof {
        key: String,
        value: String,
        signer: Vec<u8>
    },
    ChangeDoc {
        doc: IdDocument,
        signer: Vec<u8>
    },
    Recover {
        new_signer: Vec<u8>,
        new_recovery: Vec<u8>,
        signer: Vec<u8>
    }
}

impl IdentityCommand {
    pub fn handle(&self, behaviour: &mut IdentityGossipBehaviour) {
        match self {
            IdentityCommand::Get { id } => {
                let gossipsub_topic = IdentTopic::new(id.clone());
                behaviour.gossipsub.subscribe(&gossipsub_topic).unwrap();
                //behaviour.publish(id.clone(), IdentityMessage::new(IdentityCommand::Get));
            }
            IdentityCommand::SetProof { key, value, signer } => {
                /*let mut did = behaviour.identities.get()
                .did.set_proof(
                    wallet.signer_secret.clone(),
                    key.as_bytes().to_vec(),
                    value.as_bytes().to_vec(),
                );
                behaviour.publish(
                    did.id.clone(),
                    IdentityMessage::new(IdentityCommand::Post(did)),
                );*/
            }
            IdentityCommand::Recover { new_signer,new_recovery, signer } => {
               /* let mut wallet = Wallet::get(name);
                let result = wallet.did.recover(wallet.recovery_secret.clone());
                wallet.recovery_secret = result.recovery_secret;
                wallet.signer_secret = result.signer_secret;
                behaviour.publish(
                    did.id.clone(),
                    IdentityMessage::new(IdentityCommand::Post(did)),
                );*/
            }
            IdentityCommand::ChangeDoc { doc, signer } => {
                /*let mut wallet = Wallet::get(name);
                let result = wallet.did.set_doc(wallet.signer_secret.clone());
                wallet.assertion_secret = result.assertion_secret;
                wallet.authentication_secret = result.authentication_secret;
                wallet.keyagreement_secret = result.keyagreement_secret;
                Wallet::update(name, &wallet);
                behaviour.publish(
                    wallet.did.id.clone(),
                    IdentityMessage::new(IdentityCommand::Post(wallet.did)),
                );*/
            }
        }
    }
}
