pub struct IdSecret(Vec<u8>);

impl IdSecret{
   
    pub fn new() -> IdSecret {
        let mut key_data = [0u8; 32];
        let mut key_rng = thread_rng();
        key_rng.fill_bytes(&mut key_data);
        IdSecret(key_data.to_vec())
    }

    pub fn to_verification_keypair(&self) -> Keypair {
     let secret_key = SecretKey::from_bytes(self.0).unwrap();
     let public_key: PublicKey = PublicKey::from(&secret_key);
     let mut public: Vec<u8> = public_key.to_bytes().to_vec();
    let mut new_secret = self.0.clone();
    new_secret.append(&mut public);
    Keypair::from_bytes(&new_secret).unwrap()
   }

pub fn to_verification_publickey(&self) -> Vec<u8> {
    let keypair: Keypair = to_verification_keypair(secret_key);
    let public = keypair.public.to_bytes().to_vec();
    public
}

pub fn to_key_agreement_publickey(&self) -> Vec<u8> {
    let secret_data: [u8; 32] = secret.try_into().unwrap();
    let secret_key = StaticSecret::from(secret_data);
    let public_key = x25519_dalek::PublicKey::from(&secret_key);
    let public: Vec<u8> = public_key.to_bytes().to_vec();
    public
}

pub fn to_shared_secret(&self, public: &[u8]) -> x25519_dalek::SharedSecret{
    let secret_data: [u8; 32] = secret.try_into().unwrap();
    let public_data: [u8; 32] = public.try_into().unwrap();
    let sender_secret = StaticSecret::from(secret_data);
    let receiver_public = x25519_dalek::PublicKey::from(public_data);
    sender_secret.diffie_hellman(&receiver_public)
}
}