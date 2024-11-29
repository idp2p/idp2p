use libp2p::{gossipsub::TopicHash, PeerId};

pub(crate) enum IdNetworkCommand {
    Publish {
        topic: TopicHash,
        payload: Vec<u8>
    },
    Request{
        peer: PeerId,
        message_id: String
    },
    Respond {
        message_id: String,
        payload: Vec<u8>
    }
}

pub(crate) enum IdHandlerCommand {
    HandleGossipMessage(Vec<u8>),
    HandleRequest(Vec<u8>),
    HandleResponse(Vec<u8>)
}
