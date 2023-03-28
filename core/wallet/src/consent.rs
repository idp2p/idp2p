pub struct Consent {
    pub context: String,
    pub subject: String,
    pub consents: Vec<ConsentKind>,
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
