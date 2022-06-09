use idp2p_common::chrono::Utc;
use idp2p_common::multi::enc_key::Idp2pEncryptionKey;
use prost::Message;

use crate::idp2p_proto;
use crate::message::{CreateIdMessageInput, IdMessageError};

pub fn new(input: CreateIdMessageInput) -> Result<Vec<u8>, IdMessageError> {
    let auth_key = input
        .from
        .get_latest_auth_key()
        .ok_or(IdMessageError::Other)?;
    let agree_key = input
        .to
        .get_latest_agree_key()
        .ok_or(IdMessageError::Other)?;
    if input.auth_keypair.to_key() != auth_key.key {
        return Err(IdMessageError::Other);
    }
    let body = idp2p_proto::IdRawMessage {
        from: input.from.id,
        to: input.to.id,
        created_at: Utc::now().timestamp(),
        body: input.body,
    };
    let raw_bytes = body.encode_to_vec();
    let proof = input.auth_keypair.sign(&raw_bytes);
    let signed_msg = idp2p_proto::IdSignedMessage {
        signer_kid: auth_key.id,
        raw: raw_bytes,
        proof: proof,
    };
    let (shared_secret, ephemeral_key) = agree_key.key.create_shared_secret()?;
    let enc_key = Idp2pEncryptionKey::AesGcm(shared_secret);
    let enc_content = enc_key.encrypt(&signed_msg.encode_to_vec())?;
    let msg = idp2p_proto::IdEncryptedMessage {
        ephemeral_key: ephemeral_key,
        agreement_kid: agree_key.id,
        encryption_alg: enc_content.enc_alg,
        initial_vector: enc_content.initial_vector,
        cipherbody: enc_content.cipher,
    };
    Ok(msg.encode_to_vec())
}
