use serde_json::Value;
pub struct Idp2pContract {
    pub id: String,
    pub payload: Idp2pContractBody,
    pub signatures: Vec<Idp2pContractSig>,
    pub proof: String,
}

pub struct Idp2pContractBody {
    pub nonce: String, 
    pub context: String,
    pub contractors: Vec<String>,
    pub queries: Vec<Idp2pQuery>,
    pub payments: Vec<Idp2pPayment>,
    pub contract: Value,
    pub view: String,
}


pub struct Idp2pContractSig {
    pub id: String,
    pub key_id:  String, 
    pub sig: String 
}

pub struct Idp2pQuery {
    pub latest: bool,
    pub payload: Value,
}

pub struct Idp2pPayment {
    pub payer: String,   
    pub payee : String,
    pub address: String, // crypto address
    pub method: String,  // btc, eth ...
    pub amount: f64,     // payment amount
}


/*message ContractBody {
    bytes nonce = 1;
    string context = 2; // agreements
    repeated Query queries = 4;
    repeated Payment payments = 5;
    repeated bytes contractors = 6; // identity id
    bytes data = 3;
    bytes view_hash = 7;
}

message Query {
    bool latest = 1;
    string payload = 2;
}

message Payment {
    bytes payer = 1;   
    bytes payee = 2;
    string method = 3;  // btc, eth ...
    string address = 4; // crypto address
    double amount = 5;  // payment amount
}

message Idp2pContractSig {
    bytes id = 1;
    bytes key_id = 2;  
    bytes sig = 3;  
}


message Idp2pContract {
    bytes id = 1;
    bytes payload = 2;  
    repeated Idp2pContractSig signatures = 3;  
    Idp2pProof proof = 4;
}*/