extern crate alloc;
pub mod error;
pub mod inception;
pub mod did;
pub mod state;
const RELEASE_DATE: &'static str = "2022-01-01";
const VERSION: &'static str = "1.0.0";
/*mod did;
pub mod event;
pub mod state;

*/

wit_bindgen::generate!({
    world: "idp2p-id",
    generate_unused_types: true,
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
    with: {
        "idp2p:id/did": crate::did,
    }
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    #[doc = " Verifies an initial identity inception event."]
    fn verify_inception(incepiton: PersistedIdInception) -> Result<Vec<u8>, String> {
        inception::verify(&incepiton).map_err(|e| e.to_string())
    }

    #[doc = " Verifies an identity update event against the existing identity state."]
    fn verify_event(_state: Vec<u8>, _event: PersistedIdEvent) -> Result<Vec<u8>, String> {
        todo!()
    }
    
    #[doc = " Verifies an identity proof."]
    fn verify_proof(proof: IdProof,) -> Result<bool, String> {
        todo!()
    }
    /*#[doc = " Verifies an initial identity inception event."]
    fn verify_inception(inception: PersistedIdInception) -> Result<IdState, String> {
        //Ok(crate::inception::verify(&inception).map_err(|e| e.to_string())?)
    }

    #[doc = " Verifies an identity update event against the existing identity state."]
    fn verify_event(state: Vec<u8>, event: Vec<u8>) -> Result<Vec<u8>, String> {
        Ok(crate::event::verify(&state, &event[6..]).map_err(|e| e.to_string())?)
    }*/
}
