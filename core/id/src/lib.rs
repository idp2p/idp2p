pub mod event;
pub mod inception;
pub mod verify;

wit_bindgen::generate!({
    world: "idp2p-id",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

impl From<anyhow::Error> for IdError {
    fn from(e: anyhow::Error) -> Self {
        Self {
            code: IdErrorCode::Other,
            message: e.to_string(),
        }
    }
}

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
    fn verify_inception(inception: PersistedIdInception) -> Result<IdView, IdError> {
        //verify_inception(inception).map_err(|e| e.to_string())
        todo!()
    }

    fn verify_event(view: IdView, event: PersistedIdEvent) -> Result<IdView, IdError> {
        todo!()
        //verify_event(snapshot, event).map_err(|e| e.to_string())
    }
}