
use x25519_dalek::PublicKey;

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pAgreementKey {
    X25519 { public: PublicKey },
}