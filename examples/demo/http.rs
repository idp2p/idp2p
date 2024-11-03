pub mod demo_http {
    use std::sync::Arc;

    use axum::{extract::State, routing::get, Router};
    use idp2p_p2p::store::KvStore;
    struct AppState {
        kv: Arc<dyn KvStore>,
    }
    async fn root_handler(State(state): State<Arc<AppState>>) -> String {
        std::str::from_utf8(&state.kv.get("key").unwrap().unwrap())
            .unwrap()
            .to_owned()
    }
    
    fn create_router(app_state: Arc<AppState>) -> Router {
        Router::new()
            .route("/", get(root_handler))
            .with_state(app_state)
    }
    
    pub async fn create_app(kv: Arc<dyn KvStore>) {
        let app = create_router(Arc::new(AppState { kv: kv }));
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    
        axum::serve(listener, app).await.unwrap();
    }
    
}
