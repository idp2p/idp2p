
use anyhow::Result;
use ed25519_dalek::SigningKey;
use idp2p_common::{cbor, id::Id, CBOR_CODE, ED_CODE};
use idp2p_id::idp2p::id::types::{IdClaim, IdClaimValueKind, IdInception, IdSigner};
use idp2p_p2p::PersistedIdInception;
use libp2p::PeerId;
use rand::rngs::OsRng;

fn create_signer() -> IdSigner {
    let mut csprng = OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);
    let id = Id::new("signer", ED_CODE, signing_key.as_bytes())
        .unwrap()
        .to_string();
    IdSigner {
        id: id,
        public_key: signing_key.to_bytes().to_vec(),
    }
}

pub fn generate_actor(version: &str, peer: &PeerId) -> Result<PersistedIdInception> {
    let peer_claim = IdClaim {
        key: format!("/idp2p/peer/{}", peer.to_string()),
        value: IdClaimValueKind::Text(peer.to_string())
    };
    let inception = IdInception {
        timestamp: 1735689600,
        threshold: 1,
        signers: vec![create_signer()],
        next_threshold: 1,
        next_signers: vec![create_signer().id],
        claims: vec![peer_claim],
    };
    let inception_bytes = cbor::encode(&inception);
    let id = Id::new("id", CBOR_CODE, inception_bytes.as_slice()).unwrap();
    let pinception = PersistedIdInception {
        id: id.to_string(),
        version: version.to_string(),
        payload: inception_bytes,
    };
    Ok(pinception)
}

