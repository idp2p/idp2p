use std::{fs, path::Path};

use chrono::{DateTime, Utc};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::{thread_rng, RngCore};
use semver::Version;
use wasmtime::{
    component::{bindgen, Component, Linker},
    Config, Engine, Result, Store,
};

// Generate bindings of the guest and host components.
bindgen!({
    path: "core/id/wit",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IdInception {
    pub version: Version,
    pub state: Cid,
    pub config: IdConfig,
    pub timestamp: DateTime<Utc>,
    pub next_signers: Vec<IdSigner>,
}

struct MyState;

fn convert_to_component(path: impl AsRef<Path>) -> Vec<u8> {
    let bytes = &fs::read(&path).expect("failed to read input file");
    wit_component::ComponentEncoder::default()
        .module(&bytes)
        .unwrap()
        .encode()
        .unwrap()
}

fn create_random<const N: usize>() -> [u8; N] {
    let mut key_data = [0u8; N];
    let mut key_rng = thread_rng();
    key_rng.fill_bytes(&mut key_data);
    key_data
}

fn create_signer() -> (SigningKey, VerifyingKey) {
    let secret = create_random::<32>();
    let sk = SigningKey::from_bytes(&secret);
    let pk: VerifyingKey = (&sk).into();
    (sk, pk)
}

fn create_inception() -> PersistedIdInception {
    let signer = create_signer();
    let signer2 = create_signer();
    let inception = IdInception {
        version: Version::parse("0.1.0").unwrap(),
        state: Cid::default(),
        timestamp: Utc::now(),
        config: IdConfig{
            change_state_quorum: 1,
            change_config_quorum: 2,
            revoke_event_quorum: 2,
            key_reuse: true,
        },
        next_signers: vec![]
    };
    
}

fn main() -> Result<()> {
    let engine = Engine::new(Config::new().wasm_component_model(true))?;

    let component = convert_to_component("./target/wasm32-unknown-unknown/debug/wasmid.wasm");

    let component = Component::from_binary(&engine, &component)?;
    let mut store = Store::new(&engine, MyState {});
    let linker = Linker::new(&engine);
    let (idp2p, _instance) = Idp2pId::instantiate(&mut store, &component, &linker)?;
    let signer = create_signer();
    let signer2 = create_signer();
   
    let incepition_result = idp2p.call_verify_inception(&mut store, &inception_input).unwrap();
    let incepition_result = incepition_result.unwrap();

    let id = Cid::try_from(incepition_result.inception.id.clone()).unwrap();
    println!("Id: {:?}", id.to_string());

    let result = wasmid
        .call_verify_inception(&mut store, &incepition_result.inception)
        .unwrap();
    println!("Inception result: {}", result.is_ok());

    let event_result = wasmid.call_create_event(&mut store, &event_input).unwrap();
    let mut event_result = event_result.unwrap();
    let event_proof = IdEventProof {
        signer_id: incepition_result
            .clone()
            .signers
            .iter()
            .find(|s| signer.1.as_bytes().to_vec() == s.pk.clone())
            .unwrap()
            .id
            .clone(),
        signer_pub: signer.1.to_bytes().to_vec(),
        signature: signer2
            .0
            .clone()
            .sign(event_result.event.id.as_slice())
            .to_bytes()
            .to_vec(),
    };

    event_result.event.proofs.push(event_proof);
    let result2 = wasmid
        .call_verify_event(&mut store, &result.unwrap(), &event_result.event)
        .unwrap();
    println!("Event result: {}", result2.is_ok());
    Ok(())
}