#[derive(Debug, Serialize, Deserialize)]
pub struct IdEvent {
    pub version: String,
    pub timestamp: i64,
    pub previous: Cid,
    pub signers: Vec<Cid>,           
    pub next_signers: IdNextSigners,
    pub actions: Vec<IdActionKind>
}
