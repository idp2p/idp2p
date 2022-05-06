pub mod core;
pub mod json;
pub mod protobuf;
pub mod did_doc;

mod idp2p_proto {
    include!(concat!(env!("OUT_DIR"), "/idp2p.pb.rs"));
}