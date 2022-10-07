/*#[derive(Debug, StructOpt)]
#[structopt(name = "idp2pgossip", about = "Usage of idp2p gossip.")]
struct Opt {
    #[structopt(short = "p", long = "port", default_value = "43727")]
    port: u16
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    // create swarm
    // create node
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                println!("{line}");
            }
            
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                       println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(Idp2pEvent::Mdns(event)) => {
                        swarm.behaviour_mut().handle_mdns_event(event);
                    }
                    SwarmEvent::Behaviour(Idp2pEvent::Gossipsub(event)) => {
                        //swarm.behaviour_mut().handle_req_event(event)?;
                    }
                    _ => {  }
                }
            }
        }
    }
}*/
fn main(){}
pub mod error;
pub mod store;
pub mod swarm;