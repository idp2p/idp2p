use idp2p_common::error::CommonError;

use crate::IdInceptionError;

impl From<CommonError> for IdInceptionError {
    fn from(value: CommonError) -> Self {
        match value {
            CommonError::DecodeError(_) => todo!(),
            CommonError::EncodeError => todo!(),
            CommonError::InvalidPublicKey => todo!(),
            CommonError::InvalidSignature => todo!(),
            CommonError::SignatureVerifyError => todo!(),
            CommonError::MultihashError(error) => todo!(),
            CommonError::Other(error) => todo!(),
        }
    }
}