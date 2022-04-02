use crate::storage::{mysql, session};
use axum::extract::Extension;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer, Origin},
    trace::TraceLayer,
};

mod api;
mod hello_world;

pub async fn get_app() -> Router {
    let conn = mysql::get_mysql_db_conn().await;
    let session_store = session::get_session_store();

    let origins = vec![
        "https://sso.delbertbeta.life".parse().unwrap(),
        "https://sso-staging.delbertbeta.life".parse().unwrap(),
        "http://localhost:3001".parse().unwrap(),
    ];

    Router::new()
        .route("/", get(hello_world::handler))
        .route("/api/auth/register", post(api::auth::register::handler))
        .route("/api/crypto/rsa", get(api::crypto::rsa::handler))
        .layer(Extension(conn))
        .layer(Extension(session_store))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Origin::list(origins))
                .allow_methods(Any)
                .allow_headers(Any),
        )
}
