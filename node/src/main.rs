use crate::id_command::IdentityCommand;
use crate::id_swarm::create;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::Multiaddr;
use std::error::Error;
use structopt::StructOpt;
use tokio::io::{self, AsyncBufReadExt};
use qrcode::QrCode;
use qrcode::render::unicode;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "p", long = "port", default_value = "0")]
    port: u16,
    #[structopt(short = "a", long = "address")]
    dial_address: Option<String>,
    #[structopt(short = "d", long = "dir", default_value = "../target/idp2p")]
    dir: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let opt = Opt::from_args();
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    let mut swarm = create(opt.port).await?;
    let secret = idp2p_core::create_secret_key();
    let token = idp2p_core::encode(secret);
    let code = QrCode::new(token.clone()).unwrap();
    let image = code.render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    println!("{}", image);
    println!("Access token: {}", token);
    if let Some(to_dial) = opt.dial_address {
        let address: Multiaddr = to_dial.parse().expect("Invalid address.");
        match swarm.dial(address.clone()) {
            Ok(_) => println!("Dialed {:?}", address),
            Err(e) => println!("Dial {:?} failed: {:?}", address, e),
        };
    }
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<IdentityCommand>(100);
    let cmd_sender = sender.clone();
    let routes = http::routes(&token, sender.clone());
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                let cmd = id_command::handle_cmd(&line);
                if let Some(id_cmd) = cmd{
                    cmd_sender.send(id_cmd).await.unwrap();
                }
            }
            event = swarm.select_next_some() => {
                if let SwarmEvent::NewListenAddr { address, .. } = event {
                    println!("Listening on {:?}", address);
                }
            }
            message = receiver.recv() => {
                if let Some(message) = message{
                    println!("Got message: {:?}", message);
                    //let c = FileStore.get::<idp2p_core::did::Identity>("", "").await;
                    message.handle(swarm.behaviour_mut());
                }
            }
            () = warp::serve(routes.clone()).run(([127, 0, 0, 1], 3030)) => {}
        }
    }
}
pub mod http;
pub mod id_behaviour;
pub mod id_command;
pub mod id_message;
pub mod id_swarm;
pub mod file_store;
