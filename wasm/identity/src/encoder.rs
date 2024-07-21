use alloc::vec::Vec;

use crate::{
    idp2p::wasmid::model::{IdMultiSig, IdVersion},
    IdEvent, IdInception, ID_VERSION,
};

impl IdVersion {
    pub fn new() -> Self {
        Self {
            major: ID_VERSION.0,
            minor: ID_VERSION.1,
            patch: ID_VERSION.2,
        }
    }
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.major.to_be_bytes());
        bytes.extend(&self.minor.to_be_bytes());
        bytes.extend(&self.patch.to_be_bytes());
        bytes
    }
}

impl IdMultiSig {
    fn to_bytes(&self) -> Vec<u8> {
        vec![self.m, self.n]
    }
}

impl IdInception {
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        inception_to_bytes(&self.version, &self.state, &self.signers, &self.m_of_n)
    }
}

impl IdEvent {
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        event_to_bytes(
            &self.version,
            &self.previous,
            &self.state,
            &self.signers,
            &self.m_of_n,
        )
    }
}

pub fn event_to_bytes(
    version: &IdVersion,
    previous: &[u8],
    state: &[u8],
    signers: &Option<Vec<Vec<u8>>>,
    m_of_n: &Option<IdMultiSig>,
) -> anyhow::Result<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();
    // Serialize version
    bytes.extend(version.to_bytes());

    // Serialize previous
    bytes.extend((previous.len() as u64).to_be_bytes());
    bytes.extend(previous);

    // Serialize state
    bytes.extend((state.len() as u64).to_be_bytes());
    bytes.extend(state);

    // Serialize signers
    match signers {
        Some(signers) => {
            bytes.push(1);
            bytes.extend((signers.len() as u64).to_be_bytes());
            for signer in signers {
                bytes.extend((signer.len() as u64).to_be_bytes());
                bytes.extend(signer);
            }
        }
        None => bytes.push(0),
    }

    // Serialize m_of_n
    match m_of_n {
        Some(m_of_n) => {
            bytes.push(1);
            bytes.extend(m_of_n.to_bytes());
        }
        None => bytes.push(0),
    }

    Ok(bytes)
}

pub fn inception_to_bytes(
    version: &IdVersion,
    state: &[u8],
    signers: &Vec<Vec<u8>>,
    m_of_n: &IdMultiSig,
) -> anyhow::Result<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();
    // Serialize version
    bytes.extend(version.to_bytes());

    // Serialize state
    bytes.extend((state.len() as u64).to_be_bytes());
    bytes.extend(state);

    // Serialize signers
    bytes.extend((signers.len() as u64).to_be_bytes());
    for signer in signers {
        bytes.extend((signer.len() as u64).to_be_bytes());
        bytes.extend(signer);
    }

    // Serialize m_of_n
    bytes.extend(m_of_n.to_bytes());

    Ok(bytes)
}
