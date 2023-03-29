pub struct ConsentRequest {
    pub id: String,
    pub context: String,
    pub subject: String,
    pub verifier: String,
    pub body: Vec<ConsentKind>
}

// Verifier proof
// Subject proof + sdt

pub struct Consent {
    pub id: String,
    pub context: String,
    pub from: String,
    pub to: String,
    pub body: Vec<ConsentKind>,
}

pub enum ConsentKind {
    Disclosure {
        latest: bool,
        query: String,
    },
    Assertion {
        data: String,
        payment: Option<Payment>,
    },
}

pub struct Payment {
    pub address: String, // crypto address
    pub method: String,  // btc, eth ...
    pub amount: f64,     // payment amount
}
