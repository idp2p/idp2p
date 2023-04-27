use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use idp2p_common::chrono::Utc;
use idp2p_sdt::Sdt;
use serde::{Deserialize, Serialize};

use crate::{error::WalletError};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub ttl: u32,
    pub iv: Vec<u8>,
    pub cipher: Vec<u8>,
    pub session: Option<AccountSession>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct AccountSession {
    pub created_at: i64,
    pub raw: AccountRaw,
}

pub enum SessionKind {
    Admin,
    Contract(Vec<u8>)
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct AccountRaw {
    pub auth_priv_key: Vec<u8>,
    pub assert_priv_key: Vec<u8>,
    pub agree_priv_key: Vec<u8>,
    pub profile: Sdt,
    pub proofs: Vec<String>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub salt: Vec<u8>,
    pub default_user: String,
    pub accounts: HashMap<String, Account>,
}

#[derive(Debug, Clone)]
pub struct WalletStore(Arc<Mutex<Wallet>>);

impl WalletStore {
    pub fn load(&self) -> Result<(), WalletError> {
        todo!()
    }

    pub fn new_account(
        username: &str,
        pwd: &str,
        raw: AccountRaw,
    ) -> Result<(), WalletError> {
        todo!()
    }

    pub fn login(&self, pwd: &str, username: Option<&str>) -> Result<(), WalletError> {
        let mut db = self.0.lock().unwrap();
        let user = if let Some(username) = username {
            username.to_owned()
        } else {
            db.default_user.clone()
        };
        if let Some(acc) = db.accounts.get_mut(&user) {
            let createt_at = Utc::now().timestamp();
            /*let raw =
            acc.session = AccountSession{
                created_at: DateTime::timestamp(),
                raw: raw,
            };*/
        }
        //let session = AccountSession DateTime::timestamp()
        todo!()
    }

    pub fn logout() {
        // Kill store
    }
}

/*impl ContractSession {
    pub fn connect(url: String) -> Vec<u8> {
        // get request to url/did.json
        // create an ephemeral key with agreement
        // post one time message to url
        // create a window with shared secret
        todo!()
    }

    pub fn confirm() {}
}*/