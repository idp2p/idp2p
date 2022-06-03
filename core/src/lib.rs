pub mod message;
pub mod identity;
pub mod multi;
pub mod random;
pub mod serde_vec;


pub mod idp2p_proto {
    include!(concat!(env!("OUT_DIR"), "/idp2p.pb.rs"));
}

#[macro_export]
macro_rules! decode_base {
    ($s: expr) => {{
        use serde::de::Error as SerdeError;
        let data = multibase::decode(&$s).map_err(SerdeError::custom)?.1;
        Ok(data)
    }};
}

pub use libp2p;
pub use prost;
pub use thiserror;
