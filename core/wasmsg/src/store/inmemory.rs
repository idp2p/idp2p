use std::{ collections::HashMap, sync::Mutex};

use super::Store;

pub struct InMemoryStore {
    pub state: Mutex<HashMap<String, Vec<Vec<u8>>>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(HashMap::new())
        } 
    }
}

impl Store for InMemoryStore {
    fn get(&self, key: &str) -> Option<Vec<Vec<u8>>> { 
        if let Some(v) = self.state.lock().unwrap().get(key){
           return Some(v.to_vec());
        }
        None
    }

    fn put(&self, key: &str, value: Vec<Vec<u8>>) {
        let mut state = self.state.lock().unwrap();
        state.insert(key.to_owned(), value);
    }

    fn commit(&self) {}
}
