pub struct Idp2pConfig {
    id: Vec<u8>,
    hash: Idp2pHash,
    base: Idp2pBase,
    auth_keypair: Idp2pKeypair,
    agree_keypair: Idp2pKeypair 
}