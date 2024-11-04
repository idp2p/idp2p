use std::sync::Arc;

use axum::{extract::State, routing::get, Router};
use idp2p_p2p::store::KvStore;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;
pub struct AppState {
    pub kv: Arc<dyn KvStore>,
}

#[derive(OpenApi)]
struct ApiDoc;

/// Get health of the API.
#[utoipa::path(
    method(get, head),
    path = "/api/health",
    responses(
        (status = OK, description = "Success", body = str, content_type = "text/plain")
    )
)]
async fn health() -> &'static str {
    "ok"
}

async fn root_handler(State(state): State<Arc<AppState>>) -> String {
    std::str::from_utf8(&state.kv.get("key").unwrap().unwrap())
        .unwrap()
        .to_owned()
}

pub fn create_router() -> Router<Arc<AppState>> {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        .route("/key", get(root_handler))
        .split_for_parts();

    let router = router.merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api));
    router
}

pub async fn create_app(kv: Arc<dyn KvStore>, port: u16) {
    let addr = format!("0.0.0.0:{port}");
    let app = create_router();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let state = Arc::new(AppState { kv: kv });
    axum::serve(listener, app.with_state(state)).await.unwrap();
}
