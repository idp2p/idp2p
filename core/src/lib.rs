pub mod message;
pub mod identity;
pub mod store;

pub mod idp2p_proto {
    include!(concat!(env!("OUT_DIR"), "/idp2p.pb.rs"));
}

/*pub mod identity_capnp {
    include!(concat!(env!("OUT_DIR"), "/capnp/identity_capnp.rs"));
}*/

pub use libp2p;
pub use prost;
