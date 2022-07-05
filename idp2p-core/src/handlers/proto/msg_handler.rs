use crate::{
    error::Idp2pError,
    id_message::{IdMessage, MessageHandler},
    id_state::IdentityState,
    idp2p_proto,
};
use idp2p_common::{
    chrono::Utc,
    multi::{
        agreement_key::Idp2pAgreementKey, agreement_secret::Idp2pAgreementSecret,
        encryption_key::Idp2pEncryptionKey, key_secret::Idp2pKeySecret,
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
        let msg = idp2p_proto::IdEncryptedMessage::decode(msg)?;
        //let sender_agree_key = Idp2pAgreementKey::from_bytes(ephemeral_key)?;
        //let shared_key = agree_secret.to_shared_key(sender_agree_key)?;
        todo!()
    }

    fn seal_msg(
        &self,
        auth_secret: Idp2pKeySecret,
        from: IdentityState,
        to: IdentityState,
        body: &[u8],
    ) -> Result<Vec<u8>, Idp2pError> {
        let auth_key_state = from.get_latest_auth_key().ok_or(Idp2pError::Other)?;
        let agree_key_state = to.get_latest_agree_key().ok_or(Idp2pError::Other)?;
        let agree_key = Idp2pAgreementKey::from_bytes(agree_key_state.key)?;
        if auth_secret.to_key()?.to_bytes() != auth_key_state.key {
            return Err(Idp2pError::Other);
        }
        let (shared_secret, ephemeral_key) = agree_key.create_shared_secret()?;
        let proof = auth_secret.sign(&ephemeral_key)?;
        let id_msg = idp2p_proto::IdMessage {
            from: from.id,
            to: to.id,
            signer_kid: auth_key_state.id,
            proof: proof,
            body: body.to_vec(),
            created_at: Utc::now().timestamp(),
        };
        let raw_bytes = id_msg.encode_to_vec();
        let enc_key = Idp2pEncryptionKey::AesGcm(shared_secret);
        let enc_content = enc_key.encrypt(&enc_key.to_bytes(), &raw_bytes)?;
        let msg = idp2p_proto::IdEncryptedMessage {
            ephemeral_key: ephemeral_key,
            agreement_kid: agree_key_state.id,
            cipherbody: enc_content,
            encryption_key: enc_key.to_bytes(),
        };
        Ok(msg.encode_to_vec())
    }
}
