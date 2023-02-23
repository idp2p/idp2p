use idp2p_common::multi::{
    id::{Idp2pCodec, Idp2pId},
};

use crate::error::Idp2pError;

use self::{models::{MutateInput, CreateInput}, state::IdentityState, codec::proto::ProtoIdentityCodec};

#[derive(PartialEq, Debug, Clone)]
pub struct IdentityState {
    pub id: Vec<u8>,
    pub latest_event_id: Vec<u8>,
    pub owner_next_key_hash: Vec<u8>,
    pub root_next_key_hash: Vec<u8>,
    pub sdt_roots: Vec<Vec<u8>>
}

#[derive(PartialEq, Debug, Clone)]
pub struct Identity {
    pub id: Vec<u8>,
    pub microledger: Vec<u8>,
}

impl Identity {
    pub fn new() -> Result<Identity, Idp2pError> {
        // create keys
        // return keys
    }

    pub fn mutate(&mut self, sdt_root: &[u8], ) -> Result<bool, Idp2pError> {
        let id = Idp2pId::from_bytes(&self.id)?;
        
    }

    /// Verify an identity and get state of identity
    pub fn verify(&self, prev: Option<&Identity>) -> Result<IdentityState, Idp2pError> {
        let id = Idp2pId::from_bytes(&self.id)?;
        
        
    }
}

/*
{
    "keys": {
        "assertions": {
            "key1": "b12233"
        },
        "authentications": {
             "key1": "b12233"
        },
        "agreements": {
             "key1": "b12233"
        }
    },
    "personal": {
       "name": "Adem",
       "surname": "Çağlın",
       ...
    },
    "credentials": {
       "passports": {
          "1": {
             "name": "Adem",
             "birthday": "01.01.2011",
             "seals": {
                
             }
          }
       }
    }
}
*/