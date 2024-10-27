use cid::Cid;
use anyhow::{bail, Result};
use idp2p_common::{cid::CidExt, ED_CODE};
use serde::{Deserialize, Serialize};

use crate::VerificationMethod;

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
    UpdateState(Cid)
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

