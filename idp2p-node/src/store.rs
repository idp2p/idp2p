use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use idp2p_core::did::Identity;
use idp2p_core::IdStore;
use idp2p_common::serde_json;

pub struct FileIdStore {}

impl FileIdStore {
    fn get_path(&self, id: &str) -> String {
        let base_path = std::env::var("BASE_PATH").expect("$BASE_PATH is not set");
        format!("{}/identities/{}.json", base_path, id)
    }
}

impl IdStore for FileIdStore {
    fn put(&self, id: &str, value: Identity) {
        let path = self.get_path(id);
        if !std::path::Path::new(&path).exists() {
            std::fs::File::create(&path).unwrap();
        }
        let file = OpenOptions::new().write(true).open(&path).unwrap();
        serde_json::to_writer_pretty(&file, &value).unwrap();
    }

    fn get(&self, id: &str) -> Option<Identity> {
        let path = self.get_path(id);
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_path_test() {
        std::env::set_var("BASE_PATH", "idp2p");
        let store = FileIdStore {};
        let path = store.get_path( "123");
        assert_eq!(path, "idp2p/identities/123.json");
    }
}
