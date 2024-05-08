use crate::{
    error::IdError,
    id::DigestId,
    model::{IdEvent, WrappedIdEvent},
};
use alloc::vec::Vec;
use purewasm_codec::cbor::CborCodec;
use purewasm_core::{Codec, PureError, PureResult};

use crate::model::{IdCommand, IdCommandKind};

pub fn handle(input: IdCommand) -> PureResult<WrappedIdEvent> {
    let result = match input.body {
        IdCommandKind::Inception(inception) => {
            if inception.total_signer > inception.signers.len() as u8 {
                return Err(IdError::MinSignerNotMatch.into());
            }
            let result = IdEvent {
                id: inception.get_id()?,
                event_id: inception.get_id()?,
                min_signer: inception.min_signer,
                total_signer: inception.total_signer,
                signers: inception.signers,
                sdt_state: inception.sdt_state,
            };
            result
        }
        IdCommandKind::Mutation(mutation) => {
            let event_id = mutation.get_id()?;
            // Check previous context is same with current
            if mutation.payload.previous.context == input.context {
                let previous: IdEvent = CborCodec::from_bytes(&mutation.payload.previous.event)?;
                if mutation.signatures.len() < previous.min_signer as usize {
                    return Err(IdError::MinSignatureNotMatch.into());
                }
                let mut next_signers: Vec<DigestId> = Vec::new();
                let mut signers: Vec<DigestId> = previous.signers.clone();

                for sig in mutation.signatures {
                    sig.signer_id
                        .ensure(&sig.signer_pk.to_bytes())
                        .map_err(|e| PureError::new(&e))?;
                    if !signers.contains(&sig.signer_id) {
                        return Err(IdError::UnknownSigner.into());
                    }
                    if sig.next_signer_id == sig.signer_id {
                        return Err(IdError::SignerShouldBeDifferent.into());
                    }
                    let payload_bytes = CborCodec.to_bytes(&mutation.payload)?;
                    sig.signer_pk
                        .verify(&payload_bytes, sig.sig_bytes)
                        .map_err(|err| PureError::new(err))?;

                    next_signers.push(sig.next_signer_id);
                    signers.retain(|value| *value != sig.signer_id);
                }
                for (ex, next) in mutation.payload.new_signers {
                    if !signers.contains(&ex) {
                        return Err(IdError::UnknownSigner.into());
                    }
                    if ex == next {
                        return Err(IdError::SignerShouldBeDifferent.into());
                    }
                    next_signers.push(next);
                    signers.retain(|value| *value != ex);
                }
                let result = IdEvent {
                    id: previous.id,
                    event_id: event_id,
                    signers: next_signers,
                    min_signer: if let Some(ms) = mutation.payload.min_signer {
                        ms
                    } else {
                        previous.min_signer
                    },
                    total_signer: if let Some(ts) = mutation.payload.total_signer {
                        ts
                    } else {
                        previous.total_signer
                    },
                    sdt_state: if let Some(sdt) = mutation.payload.sdt_state {
                        sdt
                    } else {
                        previous.sdt_state
                    },
                };
                result
            } else {
                // There is no known version
                return Err(IdError::UnknownContext.into());
            }
        }
    };

    let bytes = CborCodec.to_bytes(&result)?;
    let wrapped = WrappedIdEvent {
        context: input.context,
        event: bytes,
    };
    Ok(wrapped)
}

#[cfg(test)]
mod tests {
    use crate::model::IdInception;

    use super::*;
    #[test]
    fn inception_test() {
        let cmd = IdCommand {
            context: DigestId::Sha256([0u8; 32]),
            body: IdCommandKind::Inception(IdInception {
                min_signer: 1,
                total_signer: 1,
                signers: vec![DigestId::Sha256([0u8; 32])],
                sdt_state: DigestId::Sha256([0u8; 32]),
            }),
        };
        let wie = handle(cmd).unwrap();
        let event: IdEvent = CborCodec::from_bytes(&wie.event).unwrap();

        eprintln!("{:?}", event);

        let r: PureResult<WrappedIdEvent> = PureResult::Err(PureError::new("NO_INPUT"));
        let b: PureResult<WrappedIdEvent> = CborCodec::from_bytes(&CborCodec.to_bytes(&r).unwrap());
        eprintln!("{:?}", b);
    }
}
