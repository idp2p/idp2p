pub enum IdAppCommand {
    IdPing(String),
    IdRotation,
    IdMessage { to: String, content: String },
}

pub enum IdAppEvent {
    IdPingNotified,
    IdPongNotified,
    IdEventNotified,
    IdMessageNotified,
    IdMessageProvided,
}

impl IdAppCommand {
    pub fn handle(&self) {
        match self {
            Self::IdPing(_) => {
                // Parse the peer_addr, peer_id and id from string
                // Subscribe to id(public channel)
                // Dial to peer_addr, peer_id
                // When connection estabilished, publish a ping to its private channel
                // Ping contains peer_addr, peer_id, id document
                // peer_addr and peer_id should be public and may be current peer or a mediator peer which subscribed to current id
            }
            Self::IdRotation => {
                // Create a rotation event and save it
                // Publish rotation event on id topic
            }
            Self::IdMessage { to, content } => {
                // Create a cid from content
                // Save message content to store as pending_messages
                // Publish to message receiver pirvate channel with provider peers
            }
        }
    }
}

impl IdAppEvent {
    pub fn handle(&self) {
        match self {
            Self::IdPingNotified => {
                // When a meet request delivered to corresponding id peer
                // Subscribe to id channels
                // Check if id exists because maybe another peer handles it 
                // Verify the id and subscribe to id channels
                // Get provided id from store
                // Generate a pong response with id doc, peer for joining channel 
                // Publish the pong message to the ping client public channel
            },
            Self::IdPongNotified => {
                // When a meet response delivered
                // Check if id exists
                // Verify the id and save
            },
            Self::IdEventNotified => {
                // When an id add an event to its document(Alice -> *)
                // Verify the id and save
            },
            Self::IdMessageNotified => {
                // When an id message notified to(Alice -> Bob)
            },
            Self::IdMessageProvided => {

            },
        }
    }
}
