#[derive(Debug, Serialize, Deserialize)]
pub struct IdEvent {
    pub version: String,
    pub timestamp: i64,
    pub previous: Cid,
    pub signers: Vec<Cid>,      // Should match m
    pub next_signers: Vec<Cid>, // Should match n
    pub sdt_root: Option<Cid>,
    pub m_of_n: Option<IdMultiSig>,
    pub key_events: Option<Vec<IdKeyEventKind>>,
}