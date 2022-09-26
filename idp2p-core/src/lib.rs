pub mod identity;
pub mod id_store;
pub mod id_state;
pub mod decoders;
pub mod error;

pub mod idp2p_proto {
    include!(concat!(env!("OUT_DIR"), "/idp2p.pb.rs"));
}

