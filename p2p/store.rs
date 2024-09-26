pub struct IdDocument {
    pub id: Cid,
}

pub trait IdStore {
    fn get(&self, id: Cid) -> IdDocument;
    fn put(&self, id: Cid, value: IdDocument);
    fn remove(&self, id: Cid);
    fn is_provider(&self, id: Cid) -> bool;
    fn is_resolved(&self, id: Cid) -> bool;
    fn add_message(&self, topic: Cid, message: IdDirectMessage);
    fn get_message(&self, topic: Cid);
}