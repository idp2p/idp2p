use idp2p_common::anyhow::*;

pub enum CommandResult{

}
pub fn handle_command(input: &str) -> Result<()> {
    let split = input.split(" ");
    let input: Vec<&str> = split.collect();
    match input[0] {
        "get" => {
            let id = input[1].to_string();
            //return Some(IdentityCommand::Get { id });
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
