use crate::IdentityError;

use self::{
    input::{ChangeInput, CreateIdentityInput, RecoverInput},
    state::IdentityState,
};
use idp2p_common::anyhow::Result;

pub mod doc;
pub mod input;
pub mod state;
pub mod protobuf;
pub mod json;

pub trait IdentityBehaviour {
    fn create(input: CreateIdentityInput) -> Result<Self>
    where
        Self: Sized;
    fn change(&mut self, input: ChangeInput) -> Result<()>;
    fn recover(&mut self, input: RecoverInput) -> Result<()>;
    fn verify(&self) -> Result<IdentityState, IdentityError>;
}
