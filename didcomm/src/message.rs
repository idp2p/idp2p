// IdGossipMessageRaw
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct IdMessageRaw {
    from: Vec<u8>,   // Message sender
    to: Vec<u8>,     // Message receiver
    kid: Vec<u8>,    // POP kid
    created_at: i64, // UTC Timestamp
    body: Vec<u8>,   // Message body
    proof: Vec<u8>,  // Proof of possession
}

// IdGossipMessageCipher
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct IdMessage {
    agreement_kid: Cid,      // Recipient agreement key id to decrypt message
    agreement_data: Vec<u8>, // Ephemeral public key (x25519) or ciphertext (Kyber)
    encryption_method: i64,  // AES-GCM (256)
    encryption_iv: Vec<u8>,  // 12 or 24 bytes initial vector
    cipherbody: Vec<u8>,     // Encrypted message body
}
