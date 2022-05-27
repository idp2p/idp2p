const IDP2P_BASE: multibase::Base = multibase::Base::Base58Btc;

pub fn decode(value: &str) -> Result<Vec<u8>, anyhow::Error> {
    let vec = multibase::decode(&value)?.1;
    Ok(vec)
}

pub fn encode(value: &[u8]) -> String {
    multibase::encode(IDP2P_BASE, value)
}



