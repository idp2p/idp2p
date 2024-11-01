#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: Vec<u8>,
    pub pk: Vec<u8>,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
}

