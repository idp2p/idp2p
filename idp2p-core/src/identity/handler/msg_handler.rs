use crate::{
    identity::{
        error::IdentityError,
        message::{IdMessage, MessageHandler},
    },
    store::IdSecret,
};

pub struct ProtoMessageHandler;

impl MessageHandler for ProtoMessageHandler {
    fn seal_msg(&self, msg: IdMessage, signer_secret: IdSecret) -> Result<Vec<u8>, IdentityError> {
        todo!()
    }

    fn decode_msg(&self, msg: &[u8], agree_secret: IdSecret) -> Result<IdMessage, IdentityError> {
        todo!()
    }
}
