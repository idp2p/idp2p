
impl NetworkBehaviourEventProcess<rendezvous::client::Event> for IdentityGossipBehaviour {
    fn inject_event(&mut self, event: rendezvous::client::Event) {
        if let rendezvous::client::Event::Discovered { registrations, .. } = event {
            for registration in registrations {
                for address in registration.record.addresses() {
                    let peer = registration.record.peer_id();
                    let p2p_suffix = Protocol::P2p(*peer.as_ref());
                    let address_with_p2p =
                        if !address.ends_with(&Multiaddr::empty().with(p2p_suffix.clone())) {
                            address.clone().with(p2p_suffix)
                        } else {
                            address.clone()
                        };
                    /*self.store.publish_event(IdentityEvent::Discovered {
                        addr: address_with_p2p,
                    });*/
                    println!("Peer {}, Addr: {}", peer, address);
                }
            }
        }
    }
}
