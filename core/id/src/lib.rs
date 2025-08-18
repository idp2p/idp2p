extern crate alloc;

pub mod error;
pub mod types;
pub mod inception;

const VALID_FROM: &str = "2026-01-01T00:00:00Z";
const VERSION: &'static str = "1.0";

wit_bindgen::generate!({
    world: "idp2p-verifier",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
    with: {
        "idp2p:id/types": crate::types,
    }
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    #[doc = " Verifies an initial identity inception event."]
    #[allow(async_fn_in_trait)]
    fn verify_inception(inception: IdEventEnvelope,) -> Result<IdState, String> {
        crate::inception::verify(&inception).map_err(|e| e.to_string())
    }
    
    #[doc = " Verifies an identity update event against the existing identity state."]
    #[allow(async_fn_in_trait)]
    fn verify_event(state:IdState,event:IdEventEnvelope,) -> Result<IdState, String> {
        todo!()
    }
    
    #[doc = " Verifies an identity proof."]
    #[allow(async_fn_in_trait)]
    fn verify_proof(signer:IdSigner,proof:IdProof,) -> Result<bool, String> {
        todo!()
    }
}
