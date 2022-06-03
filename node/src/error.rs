use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdNodeError {
    #[error("Other")]
    Other,
}
