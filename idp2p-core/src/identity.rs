use idp2p_common::{
    multi::id::{Idp2pCodec, Idp2pId},
    sdt::TrieNode,
};
use crate::{
    error::Idp2pError,
    idp2p_proto::{
        self, 
    },
};

#[derive(PartialEq, Debug, Clone)]
pub struct CreateInput {
    // Next key digest(multikey digest)
    pub root_next_pk_hash: Vec<u8>,
    // Recovery key digest(multikey digest)
    pub owner_next_pk_hash: Vec<u8>,
    pub sdt_proof: Vec<u8>
}

#[derive(PartialEq, Debug, Clone)]
pub struct Identity {
    pub id: Vec<u8>,
    pub sdt: TrieNode, // private
    pub microledger: idp2p_proto::Microledger, // public
}

impl Identity {
    pub fn new(input: CreateInput) -> Result<Identity, Idp2pError> {
        
    }

    pub fn mutate(&mut self, input: MutateInput) -> Result<bool, Idp2pError> {
        let id = Idp2pId::from_bytes(&self.id)?;
        
    }

    /// Verify an identity and get state of identity
    pub fn verify(&self, prev: Option<&Identity>) -> Result<IdentityState, Idp2pError> {
        let id = Idp2pId::from_bytes(&self.id)?;
        match id.codec {
            Idp2pCodec::Protobuf => ProtoIdentityCodec.verify(self, prev),
            Idp2pCodec::Json => todo!(),
        }
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
