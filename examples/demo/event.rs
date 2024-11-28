use libp2p::gossipsub::TopicHash;

pub(crate) enum IdNetworkEvent {
    Publish {
        topic: TopicHash,
        payload: Vec<u8>
    }
}

pub(crate) enum IdWasmEvent {
    HandleGossipMessage(Vec<u8>)
}
