use std::str::FromStr;

use idp2p_common::{
    cbor,
    error::CommonError,
    said::{Said, SaidError},
};

use crate::{
    error::IdError, idp2p::id::types::IdInception, said::Idp2pSaidKind, IdInceptionError, IdView,
    PersistedIdInception, TIMESTAMP,
};

impl PersistedIdInception {
    pub(crate) fn verify(&self) -> Result<IdView, IdInceptionError> {
        let inception: IdInception = self.try_into()?;

        // Timestamp check
        //
        if inception.timestamp < TIMESTAMP {
            todo!("timestamp error")
        }

        // Signer check
        //
        let total_signers = inception.signers.len() as u8;
        if total_signers < inception.threshold {
            todo!("threshold error")
        }
        let mut signers = vec![];
        for signer in &inception.signers {
            let signer_said = Said::from_str(signer.id.as_str())?;
            signer_said.validate(&signer.public_key)?;
            Idp2pSaidKind::from_str(&signer_said.kind)?.ensure_signer()?;
            if signers.contains(signer) {
                todo!("dublicate signers")
            }
            signers.push(signer.to_owned());
        }

        // Next Signer check
        //
        let total_next_signers = inception.next_signers.len() as u8;
        if total_next_signers < inception.next_threshold {
            todo!("next threshold error")
        }
        let mut next_signers = vec![];
        for next_signer in &inception.next_signers {
            let next_signer_said = Said::from_str(next_signer.as_str())?;
            Idp2pSaidKind::from_str(&next_signer_said.kind)?.ensure_signer()?;
            if next_signers.contains(next_signer) {
                todo!("dublicate next signers")
            }
            next_signers.push(next_signer.to_owned());
        }

        // Claims check
        //

        let mut claims = vec![];
        for claim in &inception.claims {
            let said = Said::from_str(&claim.id)?;
            Idp2pSaidKind::from_str(&said.kind)?.ensure_claim()?;
            if claims.contains(claim) {
                todo!("dublicate signers")
            }
            claims.push(claim.to_owned());
        }

        let id_view = IdView {
            id: self.id.clone(),
            event_id: self.id.clone(),
            event_timestamp: inception.timestamp,
            threshold: inception.threshold,
            signers: signers,
            next_threshold: inception.next_threshold,
            next_signers: next_signers,
            claims: claims,
        };

        Ok(id_view)
    }
}

impl TryFrom<&PersistedIdInception> for IdInception {
    type Error = IdInceptionError;

    fn try_from(value: &PersistedIdInception) -> Result<Self, Self::Error> {
        let said: Said = Said::from_str(value.id.as_str())?;
        said.validate(&value.payload)?;
        Idp2pSaidKind::from_str(&said.kind)?.ensure_id()?;
        let inception: IdInception = cbor::decode(&value.payload)?;
        Ok(inception)
    }
}

impl From<CommonError> for IdInceptionError {
    fn from(value: CommonError) -> Self {
        todo!()
    }
}

impl From<SaidError> for IdInceptionError {
    fn from(value: SaidError) -> Self {
        todo!()
    }
}

impl From<IdError> for IdInceptionError {
    fn from(value: IdError) -> Self {
        todo!()
    }
}
