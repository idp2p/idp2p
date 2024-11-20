use behaviour::{create_swarm, Idp2pBehaviourEvent};
use dotenv::dotenv;
use futures::StreamExt;
use futures::channel::oneshot;

use idp2p_p2p::{message::IdMessageHandler, store::{InMemoryKvStore, KvStore}};
use libp2p::swarm::SwarmEvent;
use std::{error::Error, sync::Arc};
use tokio::{io::AsyncBufReadExt, select};

mod behaviour;
mod http;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();

    let kv = Arc::new(InMemoryKvStore::new());
    let (sender, _) = ::channel();
    let handler = IdMessageHandler::new(kv.clone(), sender)?;

    /*task::spawn(async move {
        http::create_app(kv.clone(), 8000).await;
    });*/

    let mut swarm = create_swarm(43727)?;
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                kv.put("key", line.as_bytes()).unwrap();
                println!("Publish error: {line:?}");
            }
            event = swarm.select_next_some() => match event {

                SwarmEvent::Behaviour(Idp2pBehaviourEvent::Mdns(libp2p::mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(Idp2pBehaviourEvent::Mdns(libp2p::mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discover peer has expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(Idp2pBehaviourEvent::Gossipsub(event)) => {
                     handler.handle(event).await?;
                },
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Local node is listening on {address}");
                },
                _ => {}
            }
        }
    }
}
