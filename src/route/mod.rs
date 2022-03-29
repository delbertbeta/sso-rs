use crate::db::connection;
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
    let conn = connection::connect_db().await;

    let origins = vec![
        "https://sso.delbertbeta.life".parse().unwrap(),
        "https://sso-staging.delbertbeta.life".parse().unwrap(),
        "http://localhost:3001".parse().unwrap(),
    ];

    Router::new()
        .route("/", get(hello_world::handler))
        .route("/api/auth/register", post(api::auth::register::handler))
        .layer(Extension(conn))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Origin::list(origins))
                .allow_methods(Any),
        )
}
