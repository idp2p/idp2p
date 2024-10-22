use chrono::{DateTime, Utc};
use cid::Cid;
use anyhow::{bail, Result};
use idp2p_common::{cid::CidExt, ED_CODE};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: Vec<u8>,
    pub pk: Vec<u8>,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdActionKind {
    AddAuthentication(VerificationMethod),
    RemoveAuthentication(Cid),
    AddAssertionMethod(VerificationMethod),
    RemoveAssertionMethod(Cid),
    AddKeyAgreement(VerificationMethod),
    RemoveKeyAgreement(Cid),
    AddMediator(Cid),
    RemoveMediator(Cid),
}

impl IdActionKind {
    pub fn validate(&self) -> Result<()> {
        match self {
            IdActionKind::AddAuthentication(vm) => {
                vm.validate()?;
                let cid = Cid::from_bytes(&vm.id)?;
                match cid.codec() {
                    ED_CODE => {}
                    _ => bail!("invalid codec"),
                }
            }
            IdActionKind::AddAssertionMethod(vm) => {
                vm.validate()?;
                let cid = Cid::from_bytes(&vm.id)?;
                match cid.codec() {
                    ED_CODE => {}
                    _ => bail!("invalid codec"),
                }
            }
            IdActionKind::AddKeyAgreement(vm) => {
                vm.validate()?;
                let cid = Cid::from_bytes(&vm.id)?;
                match cid.codec() {
                    ED_CODE => {}
                    _ => bail!("invalid codec"),
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl VerificationMethod {
    pub fn validate(&self) -> Result<()> {
       todo!()
    }
}