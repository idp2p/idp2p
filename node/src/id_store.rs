use idp2p_core::did::Identity;
use async_trait::async_trait;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use anyhow::Result;

#[async_trait]
pub trait IdentityStore {
    async fn get(&self, id: &str) -> Identity;
    async fn put(did: Identity);
}

pub struct IdentityFileStore {
    base_path: String,
}

#[async_trait]
impl IdentityStore for IdentityFileStore {
    async fn get(&self, id: &str) -> Identity {
        todo!()
    }
    async fn put(did: Identity) {
        todo!()
    }
}
