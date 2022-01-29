pub struct IdWallet{
    master_salt: Vec<u8>,
    wrapped_key: Vec<u8>,
    accounts: Vec<IdAccount>
}