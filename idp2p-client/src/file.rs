use idp2p_common::anyhow::Result;
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::serde_json;
use idp2p_wallet::WalletPersister;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::str::FromStr;

use crate::{IdConfig, IdConfigResolver};

pub struct FilePersister {
    path: String,
}

impl FromStr for FilePersister {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        std::fs::create_dir_all(s.to_owned())?;
        Ok(Self { path: s.to_owned() })
    }
    type Err = Box<dyn Error>;
}

impl WalletPersister for FilePersister {
    fn wallet_exists(&self) -> bool {
        let path = format!("{}/wallet.json", self.path);
        std::path::Path::new(&path).exists()
    }
    fn get_wallet(&self) -> Result<String> {
        let path = format!("{}/wallet.json", self.path);
        let mut file = File::open(&path)?;
        let mut buff = String::new();
        file.read_to_string(&mut buff)?;
        Ok(buff)
    }
    fn persist_wallet(&self, enc_wallet: &str) -> Result<()> {
        let path = format!("{}/wallet.json", self.path);
        let file = OpenOptions::new().create_new(true).write(true).open(&path)?;
        serde_json::to_writer_pretty(&file, enc_wallet)?;
        Ok(())
    }
}

impl IdConfigResolver for FilePersister {
    fn get_config(&self, port: u16, remote: Option<String>) -> Result<IdConfig> {
        let path = format!("{}/config.json", self.path);
        if std::path::Path::new(&path).exists() {
            let file = File::open(&path)?;
            return Ok(serde_json::from_reader(file)?);
        } else {
            let config = IdConfig {
                secret: EdSecret::new().to_bytes().to_vec(),
                identities: HashMap::new(),
                listen_port: port,
                remote_addr: remote,
            };
            let file = OpenOptions::new().create_new(true).write(true).open(&path)?;
            serde_json::to_writer_pretty(&file, &config)?;
            return Ok(config);
        }
    }
}
