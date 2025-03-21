#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdRequestKind {
    MessageRequest {
        id: String,
        message_id: String
    },
    IdRequest(String)
}



