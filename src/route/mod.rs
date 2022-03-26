use crate::db::connection;
use axum::extract::Extension;
use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;

mod hello_world;

pub async fn get_app() -> Router {
    let conn = connection::connect_db().await;

    Router::new()
        .route("/", get(hello_world::handler))
        .layer(Extension(conn))
        .layer(TraceLayer::new_for_http())
}
