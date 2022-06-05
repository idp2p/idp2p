pub mod multi;
pub mod random;
pub mod serde_vec;
pub use thiserror;

#[macro_export]
macro_rules! decode_base {
    ($s: expr) => {{
        use serde::de::Error as SerdeError;
        let data = multibase::decode(&$s).map_err(SerdeError::custom)?.1;
        Ok(data)
    }};
}