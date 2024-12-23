extern crate alloc;

pub mod internal;
pub mod verify;

wit_bindgen::generate!({
    world: "idp2p-id",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

struct GuestComponent;

export!(GuestComponent);

impl Guest for GuestComponent {
    fn verify_inception(inception: PersistedIdInception) -> Result<IdView, IdInceptionError> {
        verify::verify_inception(inception)
    }

    fn verify_event(view: IdView, event: PersistedIdEvent) -> Result<IdView, IdEventError> {
        verify::verify_event(view, event)
    }
}