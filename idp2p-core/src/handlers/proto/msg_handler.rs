use crate::{
    error::Idp2pError,
    id_message::{IdMessage, MessageHandler},
    id_state::IdentityState,
    idp2p_proto,
};
use idp2p_common::{
    chrono::Utc,
    multi::{

    },
};
use prost::Message;

pub struct ProtoMessageHandler;

impl MessageHandler for ProtoMessageHandler {
    fn decode_msg(
        &self,
        agree_secret: Idp2pAgreementSecret,
        msg: &[u8],
    ) -> Result<IdMessage, Idp2pError> {
        let msg = idp2p_proto::IdGossipMessageCipher::decode(msg)?;
        let sender_agree_key = Idp2pAgreementKey::from_bytes(msg.ephemeral_key)?;
        let shared_secret = agree_secret.to_shared_key(sender_agree_key)?;
        let dec_key = Idp2pEncryptionAlg::from_bytes(msg.encryption_key)?;
        let dec_msg_bytes = dec_key.decrypt(&shared_secret, &msg.cipherbody)?;
        let id_msg = idp2p_proto::IdGossipMessageRaw::decode(&*dec_msg_bytes)?;
        Ok(id_msg.into())
    }

    fn seal_msg(
        &self,
        auth_secret: Idp2pKeySecret,
        from: IdentityState,
        to: IdentityState,
        body: &[u8],
    ) -> Result<Vec<u8>, Idp2pError> {
        let auth_key_state = from.get_latest_auth_key().ok_or(Idp2pError::Other)?.clone();
        let agree_key_state = to.get_latest_agree_key().ok_or(Idp2pError::Other)?.clone();
        let agree_key = Idp2pAgreementKey::from_bytes(agree_key_state.key)?;
        if auth_secret.to_key()?.to_bytes() != auth_key_state.key {
            return Err(Idp2pError::Other);
        }
        let (shared_secret, ephemeral_key) = agree_key.create_shared_secret()?;
        let proof = auth_secret.sign(&ephemeral_key)?;
        let id_msg = idp2p_proto::IdGossipMessageRaw {
            from: from.id,
            to: to.id,
            signer_kid: auth_key_state.id,
            proof: proof,
            body: body.to_vec(),
            created_at: Utc::now().timestamp(),
            reply_to: None
        };
        let raw_bytes = id_msg.encode_to_vec();
        let enc_key = Idp2pEncryptionAlg::new_aes_gcm();
        let enc_content = enc_key.encrypt(&shared_secret, &raw_bytes)?;
        let msg = idp2p_proto::IdGossipMessageCipher {
            ephemeral_key: ephemeral_key,
            agreement_kid: agree_key_state.id,
            cipherbody: enc_content,
            encryption_key: enc_key.to_bytes(),
        };
        Ok(msg.encode_to_vec())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use idp2p_common::random::create_random;

    use crate::id_state::{KeyState, AgreementKeyState};

    use super::*;
    #[test]
    fn enc_dec_test() -> Result<(), Idp2pError> {
        let from_auth_secret = Idp2pKeySecret::from_bytes(&create_random::<32>())?;
        let to_agree_secret = Idp2pAgreementSecret::X25519 { secret: create_random::<32>() };
        let auth_key = KeyState{
            id: from_auth_secret.to_key()?.to_id(),
            valid_at: 0,
            expired_at: None,
            key: from_auth_secret.to_key()?.to_bytes(),
        };
        let agree_key = AgreementKeyState{
            id: to_agree_secret.to_agreement_key().to_id(),
            valid_at: 0,
            expired_at: None,
            key: to_agree_secret.to_agreement_key().to_bytes(),
        };
        let from_id = IdentityState{
            id: vec![1],
            last_event_id: vec![],
            next_key_digest: vec![],
            recovery_key_digest: vec![],
            assertion_keys: vec![],
            authentication_keys: vec![auth_key],
            agreement_keys: vec![],
            proofs: HashMap::new(),
        };
        let to_id = IdentityState{
            id: vec![2],
            last_event_id: vec![],
            next_key_digest: vec![],
            recovery_key_digest: vec![],
            assertion_keys: vec![],
            authentication_keys: vec![],
            agreement_keys: vec![agree_key],
            proofs: HashMap::new(),
        };
        let msg_handler = ProtoMessageHandler{};
        let msg = msg_handler.seal_msg(from_auth_secret, from_id, to_id, &vec![0])?;
        let re_msg =msg_handler.decode_msg(to_agree_secret, &msg)?; 
        assert_eq!(re_msg.body, vec![0]);
        Ok(())
    }
}
