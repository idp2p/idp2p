use rendezvous::server::Behaviour;

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
#[behaviour(out_event = "BootstrapEvent")]
pub struct IdentityGossipBehaviour {
    identify: Identify,
    rendezvous: Behaviour,
    gossipsub: Gossipsub,
    relay: Relay,
    #[behaviour(ignore)]
    pub store: IdStore,
}