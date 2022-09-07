use crate::{
    error::Idp2pError,
    id_message::{IdMessage, MessageHandler},
    id_state::IdentityState,
    idp2p_proto,
};
use idp2p_common::{
    chrono::Utc,
    multi::{
        agreement::{Idp2pAgreementKeypair, Idp2pAgreementPublicKey},
        authentication::Idp2pAuthenticationKeypair, encryption::{Idp2pEncryptionMethod, AESGCM_CODE},
    },
};
use prost::Message;

pub struct ProtoMessageHandler;

impl MessageHandler for ProtoMessageHandler {
    fn decode_msg(
        &self,
        agree_keypair: Idp2pAgreementKeypair,
        msg: &[u8],
    ) -> Result<(IdMessage, Vec<u8>), Idp2pError> {
        let msg = idp2p_proto::IdGossipMessageCipher::decode(msg)?;
        let shared_secret = agree_keypair.resolve_shared_key(&msg.agreement_data)?;
        let dec_key = Idp2pEncryptionMethod::from_code(msg.encryption_method as u64, &msg.encryption_iv)?; 
        let dec_msg_bytes = dec_key.decrypt(&shared_secret, &msg.cipherbody)?;
        let id_msg = idp2p_proto::IdGossipMessageRaw::decode(&*dec_msg_bytes)?;
        Ok((id_msg.into(), msg.agreement_data))
    }

    fn seal_msg(
        &self,
        auth_keypair: Idp2pAuthenticationKeypair,
        from: IdentityState,
        to: IdentityState,
        body: &[u8],
    ) -> Result<Vec<u8>, Idp2pError> {
        let auth_key_state = from.get_latest_auth_key().ok_or(Idp2pError::Other)?.clone();
        let agree_key_state = to.get_latest_agree_key().ok_or(Idp2pError::Other)?.clone();
        let agree_key = Idp2pAgreementPublicKey::from_multi_bytes(&agree_key_state.key_bytes)?;
        if auth_keypair.to_public_key().to_multi_bytes() != auth_key_state.key_bytes {
            return Err(Idp2pError::Other);
        }
        let shared = agree_key.create_shared()?;
        let proof = auth_keypair.sign(&shared.data)?;
        let id_msg = idp2p_proto::IdGossipMessageRaw {
            from: from.id,
            to: to.id,
            signer_kid: auth_key_state.id,
            proof: proof,
            body: body.to_vec(),
            created_at: Utc::now().timestamp(),
            reply_to: None,
        };
        let raw_bytes = id_msg.encode_to_vec();
        let enc_key = Idp2pEncryptionMethod::new_aes_gcm();
        let Idp2pEncryptionMethod::AesGcm { iv } = &enc_key;
        let enc_content = enc_key.encrypt(&shared.secret, &raw_bytes)?;
        let msg = idp2p_proto::IdGossipMessageCipher {
            agreement_kid: agree_key_state.id,
            cipherbody: enc_content,
            agreement_data: shared.data,
            encryption_method: AESGCM_CODE as i64,
            encryption_iv: iv.to_vec(),
        };
        Ok(msg.encode_to_vec())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use idp2p_common::{random::create_random, multi::{verification::ed25519::Ed25519Keypair, agreement::x25519::X25519Keypair}};

    use crate::id_state::{AgreementPublicKeyState, AuthenticationPublicKeyState};

    use super::*;
    #[test]
    fn enc_dec_test() -> Result<(), Idp2pError> {
        let from_auth_keypair = Idp2pAuthenticationKeypair::Ed25519(
            Ed25519Keypair::from_secret_bytes(create_random::<32>()));
        let to_agree_keypair = Idp2pAgreementKeypair::X25519(
            X25519Keypair::from_secret_bytes(create_random::<32>()));
        let from_auth_pk = from_auth_keypair.to_public_key(); 
        let to_agree_pk = to_agree_keypair.to_public_key(); 
        let auth_key = AuthenticationPublicKeyState {
            id: from_auth_pk.generate_id().to_vec(),
            valid_at: 0,
            expired_at: None,
            key_bytes: from_auth_pk.to_multi_bytes(),
        };
        let agree_key = AgreementPublicKeyState  {
            id: to_agree_pk.generate_id().to_vec(),
            valid_at: 0,
            expired_at: None,
            key_bytes: to_agree_pk.to_multi_bytes(),
        };
        let from_id = IdentityState {
            id: vec![1],
            last_event_id: vec![],
            next_key_digest: vec![],
            recovery_key_digest: vec![],
            assertion_keys: vec![],
            authentication_keys: vec![auth_key],
            agreement_keys: vec![],
            proofs: HashMap::new(),
        };
        let to_id = IdentityState {
            id: vec![2],
            last_event_id: vec![],
            next_key_digest: vec![],
            recovery_key_digest: vec![],
            assertion_keys: vec![],
            authentication_keys: vec![],
            agreement_keys: vec![agree_key],
            proofs: HashMap::new(),
        };
        let msg_handler = ProtoMessageHandler {};
        let msg = msg_handler.seal_msg(from_auth_keypair, from_id, to_id, &vec![0])?;
        let (re_msg, _) = msg_handler.decode_msg(to_agree_keypair, &msg)?;
        assert_eq!(re_msg.body, vec![0]);
        Ok(())
    }
}
