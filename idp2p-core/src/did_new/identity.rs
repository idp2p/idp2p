use super::microledger::Microledger;
use idp2p_common::{ed_secret::EdSecret};
use serde::{Serialize, Deserialize};
use idp2p_common::anyhow::Result;
/// Idp2p identity 
#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct Identity {
    // Id of identity. It is CID of `MicroledgerInception` digest 
    pub id: String,
    // Identity microledger. It contains inception and eventlogs
    pub microledger: Microledger,
}

/// Convert edsecret to identity
impl TryFrom<EdSecret> for Identity {
    type Error = idp2p_common::anyhow::Error;

    fn try_from(secret: EdSecret) -> Result<Self, Self::Error> {
        let microledger: Microledger = secret.try_into()?;
        Ok(Identity{
            id: microledger.inception.get_id(),
            microledger: microledger
        })
    }
}

/// Convert bytes to identity
impl TryFrom<&[u8]> for Identity {
    type Error = idp2p_common::anyhow::Error;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Ok(rmp_serde::from_slice(bytes)?)
    }
}

#[cfg(test)]
mod tests{
    use crate::did_new::{event_log::{EventLog, EventLogPayload, EventLogPayloadChange}, Idp2pPublicKeyDigest};

    use super::*;

    #[test]
    fn create_identity_test() -> idp2p_common::anyhow::Result<()>{
        let secret = EdSecret::new();
        let mut id: Identity = secret.try_into()?;
        id.microledger.event_logs.push(EventLog{
            payload: EventLogPayload{
                previous: vec![],
                signer_key: vec![],
                next_key_digest: Idp2pPublicKeyDigest::Idp2pEd25519 { digest: vec![] },
                timestamp: 1,
                change: EventLogPayloadChange::Events { events: vec![] },
            },
            proof: vec![],
        });
        let bytes_json = idp2p_common::serde_json::to_string(&id)?;
        let bytes_rmp = rmp_serde::to_vec(&id)?;
        //eprintln!("{:?}", bytes_bincode);
        eprintln!("{}", bytes_json);
        eprintln!("JSON: {} MESSAGEPACK: {}", bytes_json.len(),  bytes_rmp.len());
        let id2: Identity = rmp_serde::from_slice(&bytes_rmp)?;
        eprintln!("{}", idp2p_common::serde_json::to_string_pretty(&id2)?);
        //let id2: Identity = rmp_serde::from_slice(&bytes)?;
        /*let bytes = bincode::serialize(&id)?;
        eprintln!("{:?}", bytes.len());
        let id2: Identity = bincode::deserialize(&bytes[..])?;*/
        Ok(())
    }
}