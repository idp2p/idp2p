pub mod error;
pub mod identity;

pub mod idp2p_proto {
    include!(concat!(env!("OUT_DIR"), "/idp2p.pb.rs"));
}

