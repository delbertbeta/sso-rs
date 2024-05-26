use crate::constants::PARSED_FRONTEND_URL;
use crate::storage::{mysql, session};
use axum::extract::Extension;
use axum::{
    routing::{get, patch, post},
    Router,
};
use http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use http::Method;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
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
        .route("/api/user", patch(api::user::patch::handler))
        .route("/api/auth/register", post(api::auth::register::handler))
        .route("/api/auth/login", post(api::auth::login::handler))
        .route("/api/auth/logout", post(api::auth::logout::handler))
        .route("/api/crypto/rsa", get(api::crypto::rsa::handler))
        .route("/api/image", post(api::image::post::handler))
        .route("/api/image/:image_id", patch(api::image::patch::handler))
        .route("/api/application", post(api::application::post::handler))
        .route("/api/application", get(api::application::get_list::handler))
        .route(
            "/api/application/:application_id",
            get(api::application::single::handler),
        )
        .route(
            "/api/application/:application_id/secrets",
            get(api::application::secret::get_list::handler),
        )
        .route(
            "/api/application/:application_id/secrets",
            post(api::application::secret::create::handler),
        )
        .layer(Extension(conn))
        .layer(Extension(session_store))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::exact(front_end_url.parse().unwrap()))
                .allow_methods(vec![Method::GET, Method::POST, Method::PATCH])
                .allow_headers(vec![AUTHORIZATION, ACCEPT, CONTENT_TYPE])
                .allow_credentials(true),
        )
}
