use dotenv::dotenv;
use idp2p_p2p::store::KvStore;
use swarm::create_swarm;
use std::{error::Error, sync::Arc};
use tokio::{io::AsyncBufReadExt, select, task};

mod http;
mod swarm;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();
    let kv = Arc::new(idp2p_p2p::store::InMemoryKvStore::new());
    let kv_clone = kv.clone();
    let _ = task::spawn(async move {
        http::create_app(kv_clone, 8000).await;
    });
    create_swarm(43727)?;
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                kv.clone().put("key", line.as_bytes()).unwrap();
                println!("Publish error: {line:?}");
            }
        }
    }
}
