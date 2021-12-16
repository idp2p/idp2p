use warp::{self, Filter};
use crate::id_command::IdentityCommand;

/*pub fn publish_message(message: String) -> Json {
    // validate secret
    // publish message
    warp::reply::json(&id)
}*/

pub fn routes(
    sender: tokio::sync::mpsc::Sender<IdentityCommand>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let send_filter = warp::any().map(move || sender.clone());
    warp::path!("resolve" / String)
        .and(send_filter.clone())
        .and_then(resolve)
}

async fn resolve(
    id: String,
    sender: tokio::sync::mpsc::Sender<IdentityCommand>,
) -> Result<impl warp::Reply, warp::Rejection> {
    sender.send(IdentityCommand::Get{id: id}).await.unwrap();
    Ok("sent on channel :)")
}
