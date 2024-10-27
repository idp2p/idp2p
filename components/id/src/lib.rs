use idp2p_core_id::inception::PersistedIdInception;

extern crate alloc;

pub const BINARY_CODE: u64 = 0x55;
pub const ID_VERSION: u64 = 0x01;

wit_bindgen::generate!({
    world: "idp2p-id"
});

struct GuestComponent;

export!(GuestComponent);

impl From<anyhow::Error> for crate::Idp2pError {
    fn from(value: anyhow::Error) -> Self {
        Self::Other(value.to_string())
    }
}

impl Guest for GuestComponent {
    fn verify_inception(inception: Vec<u8>) -> Result<Vec<u8>, Idp2pError> {
        let inception = PersistedIdInception::from_bytes(&inception)?;
        Ok(inception.verify()?)
    }

    fn verify_event(snapshot: Vec<u8>, event: Vec<u8>) -> Result<Vec<u8>, Idp2pError> {
        todo!()
    }
}