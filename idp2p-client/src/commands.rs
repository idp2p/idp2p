use crate::{swarm::IdentityClientBehaviour, persiter::FilePersister};
use idp2p_common::anyhow::Result;
use idp2p_core::IdProfile;
use idp2p_wallet::store::WalletStore;
use libp2p::{Multiaddr, PeerId, Swarm};
use std::{str::FromStr, sync::Arc};

pub enum IdCommand {
    Register {
        profile: IdProfile,
        password: String,
        listen: String,
    },
    Login(String),
    Connect(String),
    Accept(String),
    SendMessage {
        id: String,
        msg: String,
    },
}

impl IdCommand {
    pub fn handle(
        &self,
        swarm: &mut Swarm<IdentityClientBehaviour>,
        wallet_store: Arc<WalletStore<FilePersister>>,
    ) -> Result<()> {
        match &self {
            Self::Register{profile, password, listen} => {
                // save to file
                wallet_store.register(profile.clone(), password)?;
                let split: Vec<&str> = listen.split("/").collect();
                let to_dial = format!("/ip4/{}/tcp/{}", split[2], split[4]);
                let addr: Multiaddr = to_dial.parse().unwrap();
                let peer_id = PeerId::from_str(split[6])?;
                swarm.dial(addr)?;
                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
            }
            _ => {}
        }
        Ok(())
    }
}
