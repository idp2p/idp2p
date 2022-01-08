use idp2p_core::did::Identity;
use crate::file_store::FileStore;
use crate::id_command::IdentityCommand;
use warp::{self, Filter};
use std::time::Duration;
use std::thread::sleep;

pub fn routes(
    secret: String,
    sender: tokio::sync::mpsc::Sender<IdentityCommand>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let auth_filter = warp::any().and(warp::header("Authorization")).map(move | token: String| {
        token == format!("Bearer {}", secret)
    });
    let send_filter = warp::any().map(move || sender.clone());
    let resolve_routes = warp::path!("resolve" / String)
        .and(auth_filter.clone())
        .and(send_filter.clone())
        .and_then(resolve);
    let publish_routes = warp::path!("publish" / String)
        .and(auth_filter.clone())
        .and(send_filter.clone())
        .and_then(publish);
    resolve_routes.or(publish_routes)
}

async fn publish(
    message: String,
    is_authenticated: bool,
    sender: tokio::sync::mpsc::Sender<IdentityCommand>, ) -> Result<impl warp::Reply, warp::Rejection> {
    Ok("sent on channel :)")
}

async fn resolve(
    id: String,
    is_authenticated: bool,
    sender: tokio::sync::mpsc::Sender<IdentityCommand>,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("is auth: {}", is_authenticated);
    if !is_authenticated{
        return Err(warp::reject());
    }
    if let Some(identity) = FileStore.get::<Identity>("identities", &id){
        return Ok(warp::reply::json(&identity));
    }
    sender.send(IdentityCommand::Get { id: id.clone() }).await.unwrap();
    sleep(Duration::from_secs(2));
    if let Some(identity) = FileStore.get::<Identity>("identities", &id){
        return Ok(warp::reply::json(&identity));
    }
    Err(warp::reject())
}
