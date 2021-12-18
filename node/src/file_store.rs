use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;

pub struct FileStore;

impl FileStore {
    pub fn put<T: Serialize>(&self, entity: &str, id: &str, value: T) {
        let path = FileStore::get_path(entity, id);
        if !std::path::Path::new(&path).exists() {
            std::fs::File::create(&path).unwrap();
        }
        let file = OpenOptions::new().write(true).open(&path).unwrap();
        serde_json::to_writer_pretty(&file, &value).unwrap();
    }

    pub fn get<T: DeserializeOwned>(&self, entity: &str, id: &str) -> Option<T> {
        let path = FileStore::get_path(entity, id);
        let mut file = File::open(&path).unwrap();
        let mut buff = String::new();
        file.read_to_string(&mut buff).unwrap();
        Some(serde_json::from_str::<T>(&buff).unwrap())
    }

    fn get_path(entity: &str, id: &str) -> String {
        let base_path = std::env::var("BASE_PATH").expect("$BASE_PATH is not set");
        format!("{}/{}/{}.json", base_path, entity, id)
    }
}
