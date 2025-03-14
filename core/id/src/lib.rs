extern crate alloc;

mod inception;
//mod event;
mod types;
mod error;

const TIMESTAMP: i64 = 1735689600;

wit_bindgen::generate!({
    world: "idp2p-id",
    generate_unused_types: true,
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn verify_inception(inception: PersistedIdInception) -> Result<IdResult, String> {
        todo!()
        //inception.verify()
    }

    fn verify_event(state: IdState, event: PersistedIdEvent) -> Result<IdResult, String> {
        todo!()
        //let mut projection = projection;
        //event.verify(&mut projection)
    }
}


