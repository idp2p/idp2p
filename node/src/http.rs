use crate::file_store::FileStore;
use crate::id_command::IdentityCommand;
use idp2p_core::did::Identity;
use std::thread::sleep;
use std::time::Duration;
use warp::{self, Filter};

pub fn routes(
    secret: String,
    sender: tokio::sync::mpsc::Sender<IdentityCommand>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let auth_filter = warp::any()
        .and(warp::header("Authorization"))
        .map(move |token: String| token == format!("Bearer {}", secret));
    let send_filter = warp::any().map(move || sender.clone());
    let resolve_routes = warp::path!("resolve" / String)
        .and(auth_filter.clone())
        .and(send_filter.clone())
        .and_then(resolve);
    let publish_routes = warp::path!("publish")
        .and(warp::post())
        .and(json_body())
        .and(auth_filter.clone())
        .and(send_filter.clone())
        .and_then(publish);
    resolve_routes.or(publish_routes)
}

async fn publish(
    did: Identity,
    is_authenticated: bool,
    sender: tokio::sync::mpsc::Sender<IdentityCommand>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if !is_authenticated {
        return Err(warp::reject());
    }
    let cmd = IdentityCommand::Post { did: did.clone() };
    sender.send(cmd).await.unwrap();
    Ok(warp::reply::json(&did))
}

async fn resolve(
    id: String,
    is_authenticated: bool,
    sender: tokio::sync::mpsc::Sender<IdentityCommand>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if !is_authenticated {
        return Err(warp::reject());
    }
    if let Some(identity) = FileStore.get::<Identity>("identities", &id) {
        return Ok(warp::reply::json(&identity));
    }
    let cmd = IdentityCommand::Get { id: id.clone() };
    sender.send(cmd).await.unwrap();
    sleep(Duration::from_secs(2));
    if let Some(identity) = FileStore.get::<Identity>("identities", &id) {
        return Ok(warp::reply::json(&identity));
    }
    Err(warp::reject())
}

fn json_body() -> impl Filter<Extract = (Identity,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
