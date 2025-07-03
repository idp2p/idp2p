extern crate alloc;
mod error;
pub mod inception;
pub mod types;
const TIMESTAMP: i64 = 1735689600;

/*mod did;
pub mod event;
pub mod state;

const VERSION: (u16, u16) = (1, 0);*/

wit_bindgen::generate!({
    world: "idp2p-id",
    generate_unused_types: true,
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
    with: {
        "idp2p:id/id-types": crate::types,
    }
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    #[doc = " Verifies an initial identity inception event."]
    fn verify_inception(_incepiton: PersistedIdInception) -> Result<IdState, String> {
        todo!()
    }

    #[doc = " Verifies an identity update event against the existing identity state."]
    fn verify_event(_state: IdState, _event: PersistedIdEvent) -> Result<IdState, String> {
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
