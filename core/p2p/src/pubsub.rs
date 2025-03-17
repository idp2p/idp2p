#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdMessageDirection {
    From,
    To,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdPubsubMessageKind {
    // Resolve identity
    Resolve,
    // Provide identity, takes a list of providers
    Provide(Vec<String>),
    // Notify an identity event
    NotifyEvent(PersistedIdEvent),
    // Notify message(this means you have a message)
    NotifyMessage {
        id: String,
        providers: Vec<String>,
        direction: IdMessageDirection 
    }
}