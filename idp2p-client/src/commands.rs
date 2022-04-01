use idp2p_common::{anyhow::Result};
use idp2p_wallet::store::{CreateWalletInput, WalletStore};
use libp2p::{Multiaddr, PeerId, Swarm};
use std::{str::FromStr, sync::Arc};

use crate::{behaviour::IdentityClientBehaviour, persiter::FilePersister};

pub enum IdCommand {
    Listen(u16, String),
    Register(CreateWalletInput),
    Login(String),
    Connect(String),
    Accept(String),
    SendMessage { id: String, msg: String },
}

impl IdCommand {
    pub fn handle(
        &self,
        swarm: &mut Swarm<IdentityClientBehaviour>,
        wallet_store: Arc<WalletStore<FilePersister>>,
    ) -> Result<()> {
        match &self {
            Self::Listen(port, connect) => {
                // save to file
                swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", port).parse()?)?;
                let split: Vec<&str> = connect.split("/").collect();
                let to_dial = format!("/ip4/{}/tcp/{}", split[2], split[4]);
                let addr: Multiaddr = to_dial.parse().unwrap();
                let peer_id = PeerId::from_str(split[6])?;
                swarm.dial(addr)?;
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
            }
            Self::Register(input) => {
                wallet_store.register(input.clone())?;
            }
            _ => {}
        }
        Ok(())
    }
}
