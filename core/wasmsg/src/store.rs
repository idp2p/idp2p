use crate::message::WasmDocument;

pub trait Store {
    fn get(&self, key: &str) -> Option<WasmDocument> ;
    fn put(&self, key: &str, value: WasmDocument);
    fn commit(&self);
}
