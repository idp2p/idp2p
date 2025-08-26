#[derive(Debug)]
pub(crate) enum IdAppEvent {
    Resolved {
        id: String,
        peer: String
    },
    GotMessage(String),
    Other(String),
}

#[derive(Debug, Clone)]
pub enum AppState {
    WaitingForResolution,
    ReadyToMessage, // target node name
}

/// App holds the state of the application
struct App {
    /// Store
    store: Arc<InMemoryKvStore>,
    /// Current value of the input box
    input: String,
    /// History of recorded messages
    messages: Vec<String>,
    // Show help popup
    show_help_popup: bool,
    // Event sender
    network_cmd_sender: mpsc::Sender<IdNetworkCommand>,
    // Event receiver
    event_receiver: mpsc::Receiver<IdAppEvent>,
}