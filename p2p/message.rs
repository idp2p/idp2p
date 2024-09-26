#[derive(Serialize, Deserialize, Debug)]
pub struct IdDirectMessage {
    // Recipient agreement key id to decrypt message
    agreement_kid: Cid,
    // Ephemeral public key (x25519) or ciphertext (Kyber)
    agreement_data: Vec<u8>,
    // AES-GCM (256)
    encryption_method: i64,
    // 12 or 24 bytes initial vector
    encryption_iv: Vec<u8>,
    // Encrypted message body
    cipherbody: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum IdRequest {
    Register,
    Subscribe,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GossipRequest {
    Register,
    Subscribe,
}