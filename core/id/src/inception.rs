#[derive(Debug, Serialize, Deserialize)]
pub struct IdInception {
    pub version: Version,
    pub timestamp: i64,
    pub next_signers: IdNextSigners,
    pub actions: Vec<IdActionKind>,
}