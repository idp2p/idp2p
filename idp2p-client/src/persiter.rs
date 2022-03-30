use idp2p_common::anyhow::Result;
use idp2p_wallet::Persister;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

pub struct FilePersister {
    path: PathBuf,
}

impl FromStr for FilePersister {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            path: PathBuf::from_str(s)?,
        })
    }
    type Err = Box<dyn std::error::Error>;
}

impl Persister for FilePersister {
    fn exists(&self) -> bool {
        std::path::Path::new(&self.path).exists()
    }
    fn get(&self) -> Result<String> {
        let mut file = File::open(self.path.as_path())?;
        let mut buff = String::new();
        file.read_to_string(&mut buff)?;
        Ok(buff)
    }
    fn persist(&self) {
        todo!()
    }
}
