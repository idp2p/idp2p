use std::str::FromStr;

use idp2p_common::{cbor, said::Said};

use crate::{
    idp2p::id::{
        error::IdError,
        types::{IdClaim, IdClaimKind},
    },
    IdInceptionError, IdView,
};

impl IdClaim {
    pub fn validate(&self) -> Result<(), IdInceptionError> {
        let said = Said::from_str(&self.id).map_err(|e| {
            IdInceptionError::InvalidClaim(IdError {
                id: self.id.clone(),
                reason: e.to_string(),
            })
        })?;
        let kind: IdClaimKind = cbor::decode(&self.payload).map_err(|_| {
            IdInceptionError::InvalidClaim(IdError {
                id: self.id.clone(),
                reason: "invalid-claim-payload".to_string(),
            })
        })?;
        match kind {
            IdClaimKind::Mediator(_) => {
                if said.kind.as_str() != "mediator" {
                    return Err(IdInceptionError::InvalidClaim(IdError {
                        id: self.id.clone(),
                        reason: "invalid-claim-kind".to_string(),
                    }));
                }
            }
            IdClaimKind::Peer(_) => {
                if said.kind.as_str() != "peer" {
                    return Err(IdInceptionError::InvalidClaim(IdError {
                        id: self.id.clone(),
                        reason: "invalid-claim-kind".to_string(),
                    }));
                }
            }
            IdClaimKind::KeyAgreement(_) => {
                if said.kind.as_str() != "key-agreement" {
                    return Err(IdInceptionError::InvalidClaim(IdError {
                        id: self.id.clone(),
                        reason: "invalid-claim-kind".to_string(),
                    }));
                }
            }
            IdClaimKind::AssertionMethod(_) => {
                if said.kind.as_str() != "assertion-method" {
                    return Err(IdInceptionError::InvalidClaim(IdError {
                        id: self.id.clone(),
                        reason: "invalid-claim-kind".to_string(),
                    }));
                }
            }
            IdClaimKind::State(_) => {
                if said.kind.as_str() != "state" {
                    return Err(IdInceptionError::InvalidClaim(IdError {
                        id: self.id.clone(),
                        reason: "invalid-claim-kind".to_string(),
                    }));
                }
            }
        }
        Ok(())
    }
}

impl IdView {
    pub fn get_peer()
}