use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct StoredEvent {
    pub wasmid: String,
    pub payload: Vec<u8>,
}

pub struct InMemoryStore {
    pub channel: String,
    pub values: RefCell<HashMap<String, Vec<StoredEvent>>>,
}

impl InMemoryStore {
    pub fn new(ch: &str) -> Self {
        Self {
            channel: ch.to_string(),
            values: RefCell::new(HashMap::new()),
        }
    }
}

pub trait Store {
    fn get(&self, key: &str) -> Vec<StoredEvent>;
    fn put(&self, key: &str, value: Vec<StoredEvent>);
    fn commit(&self);
}

impl Store for InMemoryStore {
    fn get(&self, key: &str) -> Vec<StoredEvent> {
        todo!()
    }

    fn put(&self, key: &str, value: Vec<StoredEvent>) {
        todo!()
    }

    fn commit(&self) {
        todo!()
    }
}
