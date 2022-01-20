use didcomm_rs::Jws;
use didcomm_rs::crypto::{SignatureAlgorithm, Signer, SigningMethod, SymmetricCypherMethod, CryptoAlgorithm, Cypher};
use didcomm_rs::Jwe;
use didcomm_rs::Recepient;
use didcomm_rs::DidcommHeader;
use didcomm_rs::JwmHeader;
use crate::to_diffie_hellman;
use crate::encode;
use crate::did::Identity;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::convert::TryInto;
use x25519_dalek::{StaticSecret, PublicKey};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Message {
    #[serde(flatten)]
    pub jwm_header: JwmHeader,
    #[serde(flatten)]
    pub(crate) didcomm_header: DidcommHeader,
    pub(crate) recepients: Option<Vec<Recepient>>,
    body: String,
}

impl Message{
    pub fn new() -> Self {
        Message {
            jwm_header: JwmHeader::default(),
            didcomm_header: DidcommHeader::new(),
            recepients: None,
            body: String::default()
        }
    }
   
    pub fn from(mut self, from: &str) -> Self {
        self.didcomm_header.from = Some(String::from(from));
        self
    }
   
    pub fn to(mut self, to: &[&str]) -> Self {
        for s in to {
            self.didcomm_header.to.push(s.to_string());
        }
        while let Some(a) = self
            .didcomm_header
            .to
            .iter()
            .position(|e| e == &String::default())
        {
            self.didcomm_header.to.remove(a);
        }
        self
    }
    pub fn set_body(mut self, body: &[u8]) -> Self {
        self.body = encode(body);
        self
    }

    pub fn kid(mut self, kid: &str) -> Self {
        match &mut self.jwm_header.kid {
            Some(h) => *h = kid.into(),
            None => {
                self.jwm_header.kid = Some(kid.into());
            }
        }
        self
    }

    pub fn get_didcomm_header(&self) -> &DidcommHeader {
        &self.didcomm_header
    }

    pub fn as_jws(mut self, alg: &SignatureAlgorithm) -> Self {
        self.jwm_header.as_signed(alg);
        self
    }

    pub fn as_jwe(mut self, alg: &CryptoAlgorithm) -> Self {
        self.jwm_header.as_encrypted(alg);
        #[cfg(feature = "resolve")]
        {
            if let Some(from) = &self.didcomm_header.from {
                if let Some(document) = resolve_any(from) {
                    match alg {
                        CryptoAlgorithm::XC20P => {
                            self.jwm_header.kid = document.find_public_key_id_for_curve("X25519")
                        }
                        CryptoAlgorithm::A256GCM | CryptoAlgorithm::A256CBC => {
                            self.jwm_header.kid = document.find_public_key_id_for_curve("P-256")
                        }
                    }
                }
            }
        }
        self
    }

    pub fn seal_pre_encrypted(self, cyphertext: impl AsRef<[u8]>) -> Result<String, didcomm_rs::Error> {
        let d_header = self.get_didcomm_header();
        let mut jwe = Jwe::new(self.jwm_header.clone(), self.recepients.clone(), cyphertext);
        jwe.header.skid = Some(d_header.from.clone().unwrap_or_default());
        if !self.recepients.is_some() {
            jwe.header.kid = Some(d_header.to[0].clone());
        }
        jwe.header.skid = d_header.from.clone();
        Ok(serde_json::to_string(&jwe)?)
    }

    pub fn encrypt(
        self,
        crypter: SymmetricCypherMethod,
        encryption_key: &[u8],
    ) -> Result<String, didcomm_rs::Error> {
        let header = self.jwm_header.clone();
        let d_header = self.get_didcomm_header();
        let cyphertext = crypter(
            self.jwm_header.get_iv().as_ref(),
            encryption_key,
            serde_json::to_string(&self)?.as_bytes(),
        )?;
        let mut jwe = Jwe::new(header, self.recepients.clone(), cyphertext);
        let multi = self.recepients.is_some();
        jwe.header.skid = Some(d_header.from.clone().unwrap_or_default());
        if !multi {
            jwe.header.kid = Some(d_header.to[0].clone());
        }
        jwe.header.skid = d_header.from.clone();
        Ok(serde_json::to_string(&jwe)?)
    }

    pub fn decrypt(
        received_message: &[u8],
        decrypter: SymmetricCypherMethod,
        key: &[u8],
    ) -> Result<Self, didcomm_rs::Error> {
        let jwe: Jwe = serde_json::from_slice(received_message)?;
        if let Ok(raw_message_bytes) = decrypter(jwe.header.get_iv().as_ref(), key, &jwe.payload())
        {
            Ok(serde_json::from_slice(&raw_message_bytes)?)
        } else {
            Err(didcomm_rs::Error::PlugCryptoFailure)
        }
    }

    pub fn sign(self, signer: SigningMethod, signing_key: &[u8]) -> Result<String, didcomm_rs::Error> {
        let h = self.jwm_header.clone();
        if h.alg.is_none() {
            Err(didcomm_rs::Error::JwsParseError)
        } else {
            let payload = crate::encode(&serde_json::to_string(&self)?.as_bytes());
            let signature = signer(signing_key, &payload.as_bytes())?;
            Ok(serde_json::to_string(&Jws::new(payload, h, signature))?)
        }
    }

    pub fn verify(jws: &[u8], key: &[u8]) -> Result<Message, didcomm_rs::Error> {
        let jws: Jws = serde_json::from_slice(jws)?;
        if let Some(alg) = &jws.header.alg {
            let verifyer: SignatureAlgorithm = alg.try_into()?;
            if verifyer.validator()(key, &jws.payload.as_bytes(), &jws.signature[..])? {
                Ok(serde_json::from_slice(&multibase::decode(&jws.payload).unwrap().1)?)
            } else {
                Err(didcomm_rs::Error::JwsParseError)
            }
        } else {
            Err(didcomm_rs::Error::JwsParseError)
        }
    }
}

pub fn seal(secret: &[u8], sender: Identity, receiver: Identity, data: &str) -> Result<String, didcomm_rs::Error> {
    let rec_doc = receiver.document.unwrap();
    let rec_key_agree_pub = rec_doc.verification_method[2].bytes.clone();
    let kid = sender.document.unwrap().authentication[0].clone();
    let message = Message::new() // creating message
        .from(&sender.id) // setting from
        .to(&[&receiver.id]) // setting to
        .set_body(data.as_bytes()) // packing in some payload
        .as_jwe(&CryptoAlgorithm::XC20P) // set JOSE header for XC20P algorithm
        .kid(&kid); // set kid header
    let shared = to_diffie_hellman(secret, &rec_key_agree_pub);
    let alg = crypter_from_header(&message.jwm_header)?;
    message.encrypt(alg.encryptor(), shared.as_bytes())
}

fn crypter_from_header(header: &JwmHeader) -> Result<CryptoAlgorithm, didcomm_rs::Error> {
    match &header.alg {
        None => Err(didcomm_rs::Error::JweParseError),
        Some(alg) => alg.try_into(),
    }
}