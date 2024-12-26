use alloc::str::FromStr;

use cid::Cid;
use idp2p_common::ED_CODE;

use crate::{
    idp2p::id::types::IdInception, IdInceptionError, IdView, PersistedIdInception, Said, TIMESTAMP,
};

impl PersistedIdInception {
    pub(crate) fn verify(&self) -> Result<IdView, IdInceptionError> {
        let inception: IdInception = self.try_into()?;
        if inception.timestamp < TIMESTAMP {
            todo!("timestamp error")
        }

        // Signer threshold, codec, public bytes check
        //
        let total_next_signers = inception.next_signers.len() as u8;
        if total_next_signers < inception.next_threshold {
            todo!("threshold error")
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
        for claim in &inception.claims {
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
        let id = Said::from_str(value.id.as_str()).unwrap();
        /*id.ensure(&value.payload).should_be_id()?;
        let inception: IdInception = cbor::decode(&value.payload)?;
        Ok(inception)*/
        todo!()
    }
}

impl FromStr for Said {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}
