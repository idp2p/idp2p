use idp2p_wallet::store::WalletStore;
use idp2p_core::store::IdStore;
use std::sync::Arc;
use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent},
    mdns::{Mdns, MdnsEvent},
    NetworkBehaviour,
};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityClientEvent")]
pub struct IdentityClientBehaviour {
    pub mdns: Mdns,
    pub gossipsub: Gossipsub,
    #[behaviour(ignore)]
    pub wallet: Arc<WalletStore>,
    #[behaviour(ignore)]
    pub identities: Arc<IdStore>,
}

#[derive(Debug)]
pub enum IdentityClientEvent {
    Mdns(MdnsEvent),
    Gossipsub(GossipsubEvent),
}

impl From<MdnsEvent> for IdentityClientEvent {
    fn from(event: MdnsEvent) -> Self {
        IdentityClientEvent::Mdns(event)
    }
}
impl From<GossipsubEvent> for IdentityClientEvent {
    fn from(event: GossipsubEvent) -> Self {
        IdentityClientEvent::Gossipsub(event)
    }
}