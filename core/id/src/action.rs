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
pub struct IdService {
    id: String,
    service_endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IdActionKind {
    CreateAssertion(IdVerificationMethod),
    RevokeAssertion(Cid),
    CreateAgreement(IdVerificationMethod),
    RevokeAgreement(Cid),
    CreateService(IdService),
    RevokeService(String),
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

impl IdService {
    pub fn new(id: String, service_endpoint: String) -> Result<Self> {
        let service = Self {
            id,
            service_endpoint,
        };
        service.validate()?;
        Ok(service)
    }

    pub fn validate(&self) -> Result<()> {
        let regex_str = r"^/(dns4|ip4|ip6)/([a-zA-Z0-9\-\.]+|\d{1,3}(\.\d{1,3}){3}|[0-9a-fA-F:]+)/tcp/\d+/p2p/Qm[1-9A-HJ-NP-Za-km-z]{44}$";
        let regex = regex::Regex::new(regex_str).map_err(anyhow::Error::msg)?;
        if !regex.is_match(&self.id) {
            bail!("The service ID must match the following regex: {}", regex);
        }
        Ok(())
    }
}

impl IdActionKind {
    pub fn validate(&self) -> Result<()> {
        match self {
            IdActionKind::CreateAssertion(vm) => vm.validate()?,
            IdActionKind::CreateAgreement(vm) => vm.validate()?,
            IdActionKind::CreateService(service) => service.validate()?,
            _ => {}
        }
        Ok(())
    }
}
