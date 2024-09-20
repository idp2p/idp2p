// IdGossipMessageRaw
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct IdGossipMessageRaw {
    from: Vec<u8>,       // Message sender
    to: Vec<u8>,         // Message receiver
    kid: Vec<u8>,        // POP kid
    created_at: i64,     // UTC Timestamp
    address: String,     // Address of sender
    proof: Vec<u8>,      // Proof of possession
}

// IdGossipMessageCipher
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct IdGossipMessageCipher {
    agreement_kid: Vec<u8>,      // Recipient agreement key id to decrypt message
    agreement_data: Vec<u8>,     // Ephemeral public key (x25519) or ciphertext (Kyber)
    encryption_method: i64,      // AES-GCM (256)
    encryption_iv: Vec<u8>,      // 12 or 24 bytes initial vector
    cipherbody: Vec<u8>,         // Encrypted message body with codec
}
