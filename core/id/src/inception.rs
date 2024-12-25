use cid::Cid;
use idp2p_common::ED_CODE;

use crate::{
    idp2p::id::{error::IdInceptionErrorKind, types::IdInception},
    IdInceptionError, IdView, PersistedIdInception,
};
impl PersistedIdInception {
    pub(crate) fn verify(&self) -> Result<IdView, IdInceptionError> {
        // Decode
        //
        let inception: IdInception = self.try_into()?;
        // Timestamp check
        //

        // Signer threshold, codec, public bytes check
        //
        let total_next_signers = inception.next_signers.len() as u8;
        if total_next_signers < inception.next_threshold {
            todo!("")
        }

        for next_signer in &inception.next_signers {
            let next_signer_cid = Cid::try_from(next_signer.as_str()).unwrap();
            if next_signer_cid.codec() != ED_CODE {
                todo!("")
            }
        }

        // Next Signer threshold, codec check
        //
        let total_signers = inception.signers.len() as u8;
        if total_signers < inception.threshold {
            todo!("")
        }

        for signer in &inception.signers {
            let signer_cid = Cid::try_from(signer.id.as_str()).unwrap();
            if signer_cid.codec() != ED_CODE {
                todo!("codec error")
            }
            todo!("check public key")
        }

        // Claims check
        //
        for action in &inception.claims {
            todo!()
        }

        let id_view = IdView {
            id: self.id.clone(),
            event_id: self.id.clone(),
            event_timestamp: inception.timestamp,
            next_signers: inception.next_signers.clone(),
            signers: inception.signers,
            threshold: inception.threshold,
            claims: inception.claims,
        };

        Ok(id_view)
    }
}

impl TryFrom<&PersistedIdInception> for IdInception {
    type Error = IdInceptionError;

    fn try_from(value: &PersistedIdInception) -> Result<Self, Self::Error> {
        /*let (_, cid) =
            parse_id_with_version("id", &value.id).map_err(|_| Self::Error::InvalidId)?;
        cid.ensure(&value.payload)
            .map_err(|_| Self::Error::InvalidId)?;
        let inception: IdInception =
            cbor::decode(&value.payload).map_err(|_| Self::Error::InvalidId)?;
        Ok(inception)*/
        todo!()
    }
}
