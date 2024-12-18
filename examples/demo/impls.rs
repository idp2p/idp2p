use std::sync::Arc;

use anyhow::Result;
use cid::Cid;
use idp2p_p2p::model::{IdEntry, IdMessage, IdStore};
use wasmtime::component::Component;

use crate::store::InMemoryKvStore;

pub struct InMemoryIdStore(pub Arc<InMemoryKvStore>);

impl IdStore for InMemoryIdStore {
    async fn get_id(&self, id: &Cid) -> Result<Option<IdEntry>> {
        todo!()
    }

    async fn get_msg(&self, id: &Cid) -> Result<Option<IdMessage>> {
        todo!()
    }

    async fn set_id(&self, id: &Cid, value: &IdEntry) -> Result<()> {
        todo!()
    }

    async fn set_msg(&self, id: &Cid, value: &IdMessage) -> Result<()> {
        todo!()
    }

    async fn get_verifiers() -> Result<Vec<Component>> {
        todo!()
    }
}
