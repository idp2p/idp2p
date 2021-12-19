use crate::id_command::IdentityCommand;
use warp::{self, Filter};

pub fn routes(
    secret: &str,
    sender: tokio::sync::mpsc::Sender<IdentityCommand>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let send_filter = warp::any().map(move || sender.clone());
    let default_routes = warp::path!("resolve" / String)
        .and(send_filter.clone())
        .and_then(resolve);
    default_routes
}

pub fn publish_message(message: String) -> Result<impl warp::Reply, warp::Rejection> {
    Ok("sent on channel :)")
}

async fn resolve(
    id: String,
    sender: tokio::sync::mpsc::Sender<IdentityCommand>,
) -> Result<impl warp::Reply, warp::Rejection> {
    sender.send(IdentityCommand::Get { id: id }).await.unwrap();
    Ok("sent on channel :)")
}
