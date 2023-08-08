pub struct FilePersister { }

impl FilePersister {
    fn get_path(&self, id: &str) -> String {
        let base_path = std::env::var("BASE_PATH").expect("$BASE_PATH is not set");
        format!("{}/{}/{}.json", base_path, id)
    }
}

impl IdStore for FileStore {
    fn put(&self, id: &str, value: Identity) {
        let file_id = id[8..].to_string();
        let path = self.get_path("identities", &file_id);
        if !std::path::Path::new(&path).exists() {
            std::fs::File::create(&path).unwrap();
        }
        let file = OpenOptions::new().write(true).open(&path).unwrap();
        serde_json::to_writer_pretty(&file, &value).unwrap();
    }

    fn get(&self, id: &str) -> Option<Identity> {
        let file_id = id[8..].to_string();
        let path = self.get_path("identities", &file_id);
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

impl WalletStore for FileStore {
    fn put_wallet(&self, name: &str, value: Wallet) {
        let path = self.get_path("accounts", name);
        if !std::path::Path::new(&path).exists() {
            std::fs::File::create(&path).unwrap();
        }
        let file = OpenOptions::new().write(true).open(&path).unwrap();
        serde_json::to_writer_pretty(&file, &value).unwrap();
    }

    fn get_wallet(&self, name: &str) -> Option<Wallet> {
        let path = self.get_path("accounts", name);
        let result = File::open(&path);
        if result.is_ok() {
            let mut file = result.unwrap();
            let mut buff = String::new();
            file.read_to_string(&mut buff).unwrap();
            return Some(serde_json::from_str::<Wallet>(&buff).unwrap());
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
        let store = FileStore {};
        let path = store.get_path("identities", "123");
        assert_eq!(path, "idp2p/identities/123.json");
    }
}