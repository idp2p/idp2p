use idp2p_node::message::{IdentityMessage, IdentityMessagePayload};
use idp2p_common::anyhow::*;
use idp2p_node::behaviour::IdentityGossipBehaviour;

pub fn handle_command(input: &str, behaviour: &mut IdentityGossipBehaviour) -> Result<()> {
    let split = input.split(" ");
    let input: Vec<&str> = split.collect();
    match input[0] {
        "get" => {
            let id = input[1].to_string();
            behaviour.subscribe(id.clone())?;
            let mes_payload = IdentityMessagePayload::Get;
            let mes = IdentityMessage::new(mes_payload);
            behaviour.publish(id, mes)?;
        }
        "send-message" => {
            let message_data = input[1];
            let receiver_id = input[2].to_string();
        }
        _ => {
            println!("Unknown command");
        }
    }
    Ok(())
}
