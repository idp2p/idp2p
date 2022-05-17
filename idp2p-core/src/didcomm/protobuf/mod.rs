use crate::identity::{IdentityBehaviour, protobuf::*};
use crate::idp2p_proto::{IdRawMessage, IdSignedMessage, IdEncryptedMessage , Identity};
use idp2p_common::key::Idp2pKey;
use idp2p_common::{anyhow::Result, create_random, secret::Idp2pSecret};
use prost::Message;

pub fn create_msg(secret: Idp2pSecret, from: Identity, to: Identity, body: &[u8]) -> Result<()> {
    let to_state = from.verify()?;
    let raw = IdRawMessage {
        id: create_random::<32>().to_vec(),
        from: from.id,
        to: to.id,
        created_at: 0,
        body: vec![],
    };
    let raw_bytes = raw.encode_to_vec(); 
    let proof = secret.sign(&raw_bytes);
    let signed = IdSignedMessage{
        signer_kid : None,
        raw: raw_bytes,
        proof: proof
    };
    let signed_bytes = signed.encode_to_vec();
    //let encrypted = 
    todo!()
}
