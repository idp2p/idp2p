use idp2p_common::{error::CommonError, identifier::IdentifierError};

#[derive(Debug)]
pub struct IdError(String);

impl IdError {
    pub fn new(e: &str) -> Self {
        Self(e.to_string())
    }
}

impl From<CommonError> for IdError {
    fn from(value: CommonError) -> Self {
        todo!()
    }
}

impl From<IdentifierError> for IdError {
    fn from(value: IdentifierError) -> Self {
        todo!()
    }
}
