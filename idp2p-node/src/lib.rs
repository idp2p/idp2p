use idp2p_common::anyhow::Result;
use libp2p::Multiaddr;
use libp2p::PeerId;
use std::str::FromStr;

pub fn get_peer_info(connect: &str) -> Result<(Multiaddr, PeerId)> {
    let split: Vec<&str> = connect.split("/").collect();
    let to_dial = format!("/ip4/{}/tcp/{}", split[2], split[4]);
    let addr: Multiaddr = to_dial.parse().unwrap();
    let peer_id = PeerId::from_str(split[6])?;
    Ok((addr, peer_id))
}

pub mod behaviour;
pub mod builder;
pub mod store;
