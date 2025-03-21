#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdResponseKind {
    MessageResponse(Vec<u8>),
    IdResponse {
        inception: PersistedIdInception,
        events: HashMap<String, PersistedIdEvent>,
    }
}