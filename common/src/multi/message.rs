pub struct Idp2pMultiMessage{
    version: u64,
    codec: u64, // Codec of message
    hash_code: u64, 
    hash_size: u64,
    id: Vec<u8>,
    body: Vec<u8>
}

impl Idp2pMultiMessage{
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Idp2pMultiError>{
        
    }
    pub fn to_id(&self) -> Result<Vec<u8>>{
       
    }
}