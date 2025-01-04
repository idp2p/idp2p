extern crate alloc;

mod inception;
mod event;
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
    fn verify_inception(inception: PersistedIdInception) -> Result<IdProjection, IdInceptionError> {
        inception.verify()
    }

    fn verify_event(projection: IdProjection, event: PersistedIdEvent) -> Result<IdProjection, IdEventError> {
        let mut projection = projection;
        event.verify(&mut projection)
    }
}


