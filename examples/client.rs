use std::str::FromStr;
use idp2p_wallet::store::WalletStore;
use std::sync::Arc;
use idp2p_client::persiter::FilePersister;

fn main(){
    let persiter = FilePersister::from_str("./").expect("Invalid persister");
    let wallet_store = Arc::new(WalletStore::new(persiter));
}
