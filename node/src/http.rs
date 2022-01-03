use crate::id_command::IdentityCommand;
use warp::{self, Filter};

pub fn routes(
    secret: String,
    sender: tokio::sync::mpsc::Sender<IdentityCommand>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let auth_filter = warp::any().and(warp::header("Authorization")).map(move | token: String| {
        token == format!("Bearer {}", secret)
    });
    let send_filter = warp::any().map(move || sender.clone());
    let default_routes = warp::path!("resolve" / String)
        .and(auth_filter.clone())
        .and(send_filter.clone())
        .and_then(resolve);
    default_routes
}

pub fn publish_message(message: String, ) -> Result<impl warp::Reply, warp::Rejection> {
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
    sender.send(IdentityCommand::Get { id: id }).await.unwrap();
    Ok("sent on channel :)")
}
