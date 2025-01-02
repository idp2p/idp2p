extern crate alloc;

mod inception;
mod event;
mod validation;

const TIMESTAMP: i64 = 1735689600;
const VERSION: (u16, u16) = (0, 1);

wit_bindgen::generate!({
    world: "idp2p-id",
    generate_unused_types: true,
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn verify_inception(inception: PersistedIdInception) -> Result<IdView, IdInceptionError> {
        inception.verify()
    }

    fn verify_event(view: IdView, event: PersistedIdEvent) -> Result<IdView, IdEventError> {
        let mut view = view;
        event.verify(&mut view)
    }
}


