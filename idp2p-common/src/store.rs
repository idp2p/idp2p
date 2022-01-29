use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;

pub trait IdStore {
    fn put<T: Serialize>(&self, entity: &str, id: &str, value: T);
    fn get<T: DeserializeOwned>(&self, entity: &str, id: &str) -> Option<T>;
}

pub struct FileStore {}

impl FileStore{
    fn get_path(&self, entity: &str, id: &str) -> String {
        let base_path = std::env::var("BASE_PATH").expect("$BASE_PATH is not set");
        format!("{}/{}/{}.json", base_path, entity, id)
    }
}

impl IdStore for FileStore{
    fn put<T: Serialize>(&self, entity: &str, id: &str, value: T) {
        let path = self.get_path(entity, id);
        if !std::path::Path::new(&path).exists() {
            std::fs::File::create(&path).unwrap();
        }
        let file = OpenOptions::new().write(true).open(&path).unwrap();
        serde_json::to_writer_pretty(&file, &value).unwrap();
    }

    fn get<T: DeserializeOwned>(&self, entity: &str, id: &str) -> Option<T> {
        let path = self.get_path(entity, id);
        let result = File::open(&path);
        if result.is_ok(){
            let mut file = result.unwrap();
            let mut buff = String::new();
            file.read_to_string(&mut buff).unwrap();
            return Some(serde_json::from_str::<T>(&buff).unwrap())
        }
        None
    }
}
