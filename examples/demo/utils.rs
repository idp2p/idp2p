use std::collections::HashMap;

use anyhow::Result;
use chrono::Utc;
use cid::Cid;
use ed25519_dalek::SigningKey;
use idp2p_common::{cbor, cid::CidExt, ED_CODE};
use idp2p_id::{internal::IdInception, IdConfig, IdMultisig};
use idp2p_p2p::{model::PersistedId, PersistedIdInception};
use libp2p::PeerId;
use rand::rngs::OsRng;

pub fn generate_id(mediator: &PeerId) -> Result<PersistedId> {
    let mut csprng = OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);
    let inception = generate(&signing_key.to_bytes(), &mediator.to_string())?;
    let payload = cbor::encode(&inception)?;
    let cid = Cid::create(0x01, payload.as_slice())?;
    let persisted_id = PersistedId {
        id: cid.to_bytes(),
        version: 1,
        inception: PersistedIdInception {
            id: cid.to_bytes(),
            payload: payload,
        },
        events: HashMap::new(),
    };
    Ok(persisted_id)
}

pub fn generate(signer: &[u8], mediator: &str) -> anyhow::Result<IdInception> {
    let state = cid::Cid::default();
    let signer = Cid::create(ED_CODE, signer)?;
    let next_signers = vec![signer];

    let inception = IdInception {
        config: IdConfig {
            multisig: IdMultisig::OneOfOne,
            key_reuse: true,
        },
        state,
        timestamp: Utc::now(),
        next_signers,
        mediators: vec![mediator.to_owned()],
    };

    Ok(inception)
}
/*
fn get_component(&self, version: u64) -> Result<(Idp2pId, Store<()>)> {
    let mut store = Store::new(&self.engine, ());
    let component = self
        .id_components
        .lock()
        .unwrap()
        .get(&version)
        .unwrap()
        .clone();
    let (id, _) = Idp2pId::instantiate(&mut store, &component, &Linker::new(&self.engine))?;
    Ok((id, store))
}

fn verify_inception(&self, version: u64, inception: &PersistedIdInception) -> Result<IdView> {
    let (verifier, mut store) = self.get_component(version)?;
    let view = verifier.call_verify_inception(&mut store, inception)??;
    Ok(view)
}

fn verify_event(&self, version: u64, view: &IdView, event: &PersistedIdEvent) -> Result<IdView> {
    let (verifier, mut store) = self.get_component(version)?;
    let view = verifier.call_verify_event(&mut store, view, event)??;
    Ok(view)
}


        let engine = Engine::new(Config::new().wasm_component_model(true))?;

        let components = HashMap::new();

        let id_components = Arc::new(Mutex::new(components));
*/
