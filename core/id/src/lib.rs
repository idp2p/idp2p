pub mod event;
pub mod inception;
pub mod verify;

wit_bindgen::generate!({
    world: "idp2p-id",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

impl IdMultisig {
    pub fn total_signers(&self) -> u16 {
        match self {
            IdMultisig::OneOfOne => 1,
            IdMultisig::OneOfTwo => 2,
            IdMultisig::TwoOfTree => 3,
            IdMultisig::ThreeOfFive => 5,
        }
    }

    pub fn get_min_signers(&self) -> u16 {
        match self {
            IdMultisig::OneOfOne => 1,
            IdMultisig::OneOfTwo => 1,
            IdMultisig::TwoOfTree => 2,
            IdMultisig::ThreeOfFive => 3,
        }
    }
}

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