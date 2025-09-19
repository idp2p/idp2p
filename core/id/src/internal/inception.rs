use super::{claim::IdClaimCreateEvent, signer::IdSigner};
use alloc::collections::BTreeSet;
use alloc::string::String;
use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdInception {
    pub version: String,
    pub patch: Cid,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub prior_id: Option<String>,
    pub threshold: u8,
    pub next_threshold: u8,
    pub signers: BTreeSet<IdSigner>,
    pub next_signers: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty", default)]
    pub claims: BTreeSet<IdClaimCreateEvent>,
}

/*#[cfg(test)]
mod tests {
    use crate::types::IdProof;
    use ed25519_dalek::{SigningKey, VerifyingKey, ed25519::signature::SignerMut};
    use idp2p_common::{CBOR_CODE, ED_CODE};
    use rand::rngs::OsRng;

    use super::*;

    fn create_signer() -> (String, VerifyingKey, SigningKey) {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let id = Cid::create(ED_CODE, verifying_key.as_bytes())
            .unwrap()
            .to_string();
        (id, verifying_key, signing_key)
    }

    #[test]
    fn test_verify_inception() {
        let signer1 = create_signer();
        let signer2 = create_signer();
        let next_signer1 = create_signer();
        let next_signer2 = create_signer();

        let mut signers = BTreeSet::new();
        let mut next_signers = BTreeSet::new();

        signers.insert(IdSigner {
            id: signer1.0.clone(),
            public_key: signer1.1.as_bytes().to_vec(),
        });
        signers.insert(IdSigner {
            id: signer2.0.clone(),
            public_key: signer2.1.as_bytes().to_vec(),
        });

        next_signers.insert(next_signer1.0);
        next_signers.insert(next_signer2.0);
        let inception = IdInception {
            version: "1.0".into(),
            patch: Cid::default(),
            timestamp: Utc::now().timestamp(),
            prior_id: None,
            threshold: 1,
            next_threshold: 1,
            signers: signers.clone(),
            next_signers: next_signers,
            claims: BTreeSet::new(),
        };
        let inception_bytes = cbor::encode(&inception);
        let created_at = Utc::now();

        let proof1 = IdProof {
            key_id: signer1.0.clone(),
            created_at: created_at.clone().to_rfc3339(),
            signature: signer1.clone().2.sign(&inception_bytes).to_vec(),
        };
        let proof2 = IdProof {
            key_id: signer2.0.clone(),
            created_at: created_at.clone().to_rfc3339(),
            signature: signer2.clone().2.sign(&inception_bytes).to_vec(),
        };
        let id = Cid::create(CBOR_CODE, inception_bytes.as_slice())
            .unwrap()
            .to_string();
        eprintln!("ID: {}", id.to_string());
        let pinception = IdEventReceipt {
            id: id.to_string(),
            version: "1.0".into(),
            created_at: Utc::now().to_rfc3339(),
            payload: inception_bytes,
            proofs: vec![proof1, proof2],
            external_proofs: Vec::new(),
        };
        let result = verify(&pinception);
        eprintln!("Result: {:#?}", result);
        assert!(result.is_ok());
        let result: String = serde_json::to_string_pretty(&result.unwrap()).unwrap();
        eprintln!("Result: {result}");
    }
}*/
