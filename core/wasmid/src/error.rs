use purewasm_core::PureError;

pub enum IdError {
    UnknownContext,
    UnknownSigner,
    MinSignatureNotMatch,
    MinSignerNotMatch,
    SignerShouldBeDifferent
}

impl Into<PureError> for IdError {
    fn into(self) -> PureError {
        match self {
            IdError::UnknownContext => PureError::new("UNKNOWN_CONTEXT"),
            IdError::UnknownSigner => PureError::new("UNKNOWN_SIGNER"),
            IdError::MinSignerNotMatch => PureError::new("MIN_SIGNER_NOT_MATCH"),
            IdError::MinSignatureNotMatch => PureError::new("MIN_SIGNATURE_NOT_MATCH"),
            IdError::SignerShouldBeDifferent => PureError::new("SIGNER_SHOULD_BE_DIFFERENT"),
        }
    }
}

