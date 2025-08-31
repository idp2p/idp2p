use crate::{
    exports::idp2p::core::id_verifier::Guest,
    types::{IdEventReceipt, IdProofReceipt, IdSigner, IdState, Idp2pError},
};

extern crate alloc;

pub mod types;
pub mod verifier;

const VALID_FROM: &str = "2025-01-01T00:00:00Z";
const VERSION: &'static str = "1.0";

wit_bindgen::generate!({
    world: "idp2p-id-verifier",
    path: "../../wit",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
    with: {
        "idp2p:core/types": crate::types,
    }
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    #[doc = " Verifies an initial identity inception event."]
    #[allow(async_fn_in_trait)]
    fn verify_inception(inception: IdEventReceipt) -> Result<IdState, Idp2pError> {
        //crate::inception::verify(&inception).map_err(|e| e.to_string())
        todo!()
    }

    #[doc = " Verifies an identity update event against the existing identity state."]
    #[allow(async_fn_in_trait)]
    fn verify_event(state: IdState, event: IdEventReceipt) -> Result<IdState, Idp2pError> {
        todo!()
    }

    #[doc = " Verifies an identity proof."]
    #[allow(async_fn_in_trait)]
    fn verify_proof(proof: IdProofReceipt, signer: IdSigner, data: Vec<u8>) -> Result<bool, Idp2pError> {
        todo!()
    }
}
