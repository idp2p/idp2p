use dotenv::dotenv;
use tokio::task;
use std::{error::Error, sync::Arc};

mod http;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();
    let kv = Arc::new(idp2p_p2p::store::InMemoryKvStore::new());
    let _ = task::spawn(async move {
        //http::demo_http::create_app(kv).await;
    });
    println!("Hello, world!");
    Ok(())
}