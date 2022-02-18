use idp2p_node::store::IdStore;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use idp2p_core::did::Identity;
use idp2p_common::serde_json;
use serde::{Deserialize, Serialize};

pub struct FileStore {}

impl FileStore {
    fn get_path(&self, entity: &str, id: &str) -> String {
        let base_path = std::env::var("BASE_PATH").expect("$BASE_PATH is not set");
        format!("{}/{}/{}.json", base_path, entity, id)
    }
}

impl IdStore for FileStore {
    fn put(&self, id: &str, value: Identity) {
        let path = self.get_path("identities", id);
        if !std::path::Path::new(&path).exists() {
            std::fs::File::create(&path).unwrap();
        }
        let file = OpenOptions::new().write(true).open(&path).unwrap();
        serde_json::to_writer_pretty(&file, &value).unwrap();
    }

    fn get(&self, id: &str) -> Option<Identity> {
        let path = self.get_path("identities", id);
        let result = File::open(&path);
        if result.is_ok() {
            let mut file = result.unwrap();
            let mut buff = String::new();
            file.read_to_string(&mut buff).unwrap();
            return Some(serde_json::from_str::<Identity>(&buff).unwrap());
        }
        None
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SwarmAccount{
    pub name: String,
    pub id: String,
    pub authentication_secret: Vec<u8>,
    pub agreement_secret: Vec<u8>,
}

pub trait AccountStore {
    fn put(&self, key: &str, value: SwarmAccount);
    fn get(&self, key: &str) -> Option<SwarmAccount>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_path_test() {
        std::env::set_var("BASE_PATH", "idp2p");
        let store = FileStore {};
        let path = store.get_path("identities", "123");
        assert_eq!(path, "idp2p/identities/123.json");
    }
}
