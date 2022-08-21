pub trait DidBehaviour {
    fn to_did_scheme(&self) -> String;
}

impl DidBehaviour for Idp2pAgreementKeyCode{
    fn to_did_scheme(&self) -> String{
        match self {
            Self::X25519 { public: _ } => "X25519VerificationKey2020".to_string(),
            Self::Kyber512 { public } => todo!(),
        }
    }
}