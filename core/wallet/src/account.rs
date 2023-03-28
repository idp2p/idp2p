use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use idp2p_common::chrono::Utc;
use idp2p_sdt::Sdt;
use serde::{Serialize, Deserialize};

use crate::{error::WalletError, consent::Consent};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub ttl: u32,
    pub iv: Vec<u8>,
    pub cipher: Vec<u8>,
    pub session: Option<AccountSession>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct AccountResult {
    pub id: String,
    pub ttl: u32,
    pub session: Option<i64>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct AccountSession {
    pub created_at: i64,
    pub raw: AccountRaw,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct AccountRaw {
    pub auth_priv_key: Vec<u8>,
    pub assert_priv_key: Vec<u8>,
    pub agree_priv_key: Vec<u8>,
    pub profile: Sdt,
    pub proofs: Vec<String>
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub salt: Vec<u8>,
    pub accounts: HashMap<String, Account>
}

#[derive(Debug, Clone)]
pub struct WalletStore(Arc<Mutex<Wallet>>);

impl WalletStore {
    pub fn new(pwd: &str) -> Result<(), WalletError> {
        // Create a wallet
        todo!()
    }

    pub fn load(&self) -> Result<Vec<String>, WalletError> {
        todo!()
    }

    pub fn new_account(
        name: &str,
        pwd: &str,
        raw: AccountRaw,
    ) -> Result<AccountResult, WalletError> {
        todo!()
    }

    pub fn login(&self, name: &str, pwd: &str) -> Result<AccountResult, WalletError> {
        let mut db = self.0.lock().unwrap();
        if let Some(acc) = db.accounts.get_mut(name) {
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

impl AccountSession {
    pub fn connect(url: String) -> Vec<u8> {
        // get request to url/did.json
        // create an ephemeral key with agreement
        // post one time message to url
        // create a window with shared secret
        todo!()
    }

    pub fn assert(consent: Consent) {
        
    }
}
