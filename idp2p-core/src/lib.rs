use id_message::MessageHandler;
use identity::IdentityHandler;
use idp2p_common::multi::id::Idp2pCodec;

pub mod identity;
pub mod id_store;
pub mod id_state;
pub mod id_message;
pub mod handlers;
pub mod error;

pub trait HandlerResolver {
    fn resolve_msg_handler(&self) -> Box<dyn MessageHandler>;
    fn resolve_id_handler(&self) -> Box<dyn IdentityHandler>;
}

impl HandlerResolver for Idp2pCodec{
    fn resolve_msg_handler(&self) -> Box<dyn MessageHandler> {
        match self{
            Idp2pCodec::Protobuf => todo!(),
            Idp2pCodec::Json => todo!(),
        }
    }

    fn resolve_id_handler(&self) -> Box<dyn IdentityHandler> {
        match self{
            Idp2pCodec::Protobuf => todo!(),
            Idp2pCodec::Json => todo!(),
        }
    }
}

pub mod idp2p_proto {
    include!(concat!(env!("OUT_DIR"), "/idp2p.pb.rs"));
}

