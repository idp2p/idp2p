use anyhow::{bail, Result};
use cid::Cid;
use idp2p_utils::cid::CidExt;
use serde::{Deserialize, Serialize};
use alloc::vec::Vec;

#[derive(Debug, Serialize, Deserialize)]
pub struct IdVerificationMethod {
    pub id: Cid,
    pub pk: Vec<u8>,
    pub valid_from: i64,
    pub valid_until: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IdActionKind {
    CreateAssertion(IdVerificationMethod),
    RevokeAssertion(Cid),
    CreateAgreement(IdVerificationMethod),
    RevokeAgreement(Cid),
    CreateMediator(Cid),
    RevokeMediator(Cid),
}

impl IdVerificationMethod {
    pub fn new(id: Cid, pk: Vec<u8>, valid_from: i64, valid_until: i64) -> Result<Self> {
        let verification_method = Self {
            id,
            pk,
            valid_from,
            valid_until,
        };
        verification_method.validate()?;
        Ok(verification_method)
    }

    pub fn validate(&self) -> Result<()> {
        self.id.ensure(self.pk.as_slice())?;
        if self.valid_from > self.valid_until {
            bail!("The verification method `valid_from` time must be less than or equal to the verification method `valid_until` time.");
        }
        Ok(())
    }
}


impl IdActionKind {
    pub fn validate(&self) -> Result<()> {
        match self {
            IdActionKind::CreateAssertion(vm) => vm.validate()?,
            IdActionKind::CreateAgreement(vm) => vm.validate()?,
            _ => {}
        }
        Ok(())
    }
}
