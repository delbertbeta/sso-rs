use crate::constants::PARSED_FRONTEND_URL;
use crate::storage::{mysql, session};
use axum::extract::Extension;
use axum::routing::patch;
use axum::{
    routing::{get, post},
    Router,
};
use http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use tower_http::{
    cors::{Any, CorsLayer, Origin},
    trace::TraceLayer,
};

mod api;
mod hello_world;

pub async fn get_app() -> Router {
    let conn = mysql::get_mysql_db_conn().await;
    let session_store = session::get_session_store();

    let front_end_url = PARSED_FRONTEND_URL.to_string();
    let front_end_url = front_end_url.trim_end_matches("/");

    Router::new()
        .route("/", get(hello_world::handler))
        .route("/api/user", get(api::user::user::handler))
        .route("/api/auth/register", post(api::auth::register::handler))
        .route("/api/auth/login", post(api::auth::login::handler))
        .route("/api/auth/logout", post(api::auth::logout::handler))
        .route("/api/crypto/rsa", get(api::crypto::rsa::handler))
        .route("/api/image", post(api::image::post::handler))
        .route("/api/image/:image_id", patch(api::image::patch::handler))
        .layer(Extension(conn))
        .layer(Extension(session_store))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Origin::exact(front_end_url.parse().unwrap()))
                .allow_methods(Any)
                .allow_headers(vec![AUTHORIZATION, ACCEPT, CONTENT_TYPE])
                .allow_credentials(true),
        )
}
