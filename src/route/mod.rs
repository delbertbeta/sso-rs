use crate::db::connection;
use axum::extract::Extension;
use axum::{routing::get, Router};

mod hello_world;

pub async fn get_app() -> Router {
    let conn = connection::connect_db().await;

    Router::new()
        .route("/", get(hello_world::handler))
        .layer(Extension(conn))
}
