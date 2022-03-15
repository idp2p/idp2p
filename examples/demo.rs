use idp2p_wallet::wallet::WalletStore;
use idp2p_wallet::wallet::Wallet;
use idp2p_node::store::IdStore;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use idp2p_core::did::Identity;
use idp2p_common::serde_json;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "p", long = "port", default_value = "43727")]
    port: u16,
}

fn get_command(input: &str) -> Option<IdCommand> {
    let split = input.split(" ");
    let input: Vec<&str> = split.collect();
    match input[0] {
        "create" => {
            // create alice
            return Some(IdCommand::Create(input[1].to_owned()));
        }
        "set-document" => {
            // set-document for alice
            return Some(IdCommand::SetDocument);
        }
        "get" => {
            // get <id>
            return Some(IdCommand::Get(input[1].to_owned()));
        }
        "resolve" => {
            // get <id>
            return Some(IdCommand::Resolve(input[1].to_owned()));
        }
        "send" => {
            // send <message> to <id>
            return Some(IdCommand::SendJwm {
                to: input[3].to_owned(),
                message: input[1].to_owned(),
            });
        }
        _ => {
            return None;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let opt = Opt::from_args();
    let base_path = format!("../target/{}", opt.port);
    std::env::set_var("BASE_PATH", base_path.clone());
    let acc_path = format!("{}/accounts", base_path);
    std::fs::create_dir_all(acc_path).unwrap();
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    let (tx, mut rx) = channel::<IdentityEvent>(100);
    let options = SwarmOptions {
        addr: opt.address,
        port: opt.port,
        store: IdStore::new(tx.clone()),
    };
    let mut swarm = build_swarm(options).await?;
    
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                if let Some(cmd) = get_command(&line){
                    cmd.handle(swarm.behaviour_mut())?;
                }
            }
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::ConnectionEstablished{peer_id, ..} => {
                        println!("Established {:?}", peer_id);
                    }
                    other => {
                        println!("Unhandled {:?}", other);
                    }
                }
            }
            event = rx.recv() => {
                if let Some(event) = event{
                    match event{
                        IdentityEvent::ReceivedJwm {id, jwm} => {
                            let mes = handle_jwm(&id, &jwm, swarm.behaviour_mut())?;
                            println!("{mes}");
                        }
                        _ => println!("{:?}", event)
                    }
                }
            }
        }
    }
}

