pub struct IdDocument {
    pub id: Cid,
}

pub trait IdStore {
    fn get(&self, id: Cid) -> IdDocument;
    fn put(&self, id: Cid, value: IdDocument);
    fn remove(&self, id: Cid);
}