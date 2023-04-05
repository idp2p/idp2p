pub struct ContractRequest {
    pub id: String, 
    pub context: String,
    pub subject: String,
    pub verifier: String,
    pub disclosures: Vec<Disclosure>,
    pub contract: Value,
    pub payments: Vec<Payment>,
    pub view: ConsentView,
}

// Verifier proof
// Subject proof 
// Information(Sdt)

pub struct Disclosure {
    latest: bool,
    query: String,
}

pub struct Payment {
    pub address: String, // crypto address
    pub method: String,  // btc, eth ...
    pub amount: f64,     // payment amount
}
