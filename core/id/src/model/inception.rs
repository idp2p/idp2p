#[derive(Debug, Serialize, Deserialize)]
pub struct IdInception {
    pub version: String,
    pub timestamp: i64,
    pub m_of_n: IdMultiSig,
    pub next_signers: Vec<Cid>, // Should match n
    pub sdt_root: Cid,
    pub key_events: Vec<IdKeyEventKind>,
}