use crate::constants::ENVS;
use crate::constants::PARSED_FRONTEND_URL;
use async_redis_session::RedisSessionStore;
use aws_sdk_s3::Client;
use axum::extract::Extension;
use axum::{
    routing::{get, patch, post},
    Router,
};
use http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use http::Method;
use migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

mod api;
mod hello_world;

pub async fn get_app(conn: DatabaseConnection, session_store: RedisSessionStore, s3_client: Client) -> Router {
    let is_prod = ENVS.prod;

    if is_prod {
        Migrator::up(&conn, None).await.expect("Migrator up failed");
    }

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
        .layer(Extension(s3_client))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::exact(front_end_url.parse().unwrap()))
                .allow_methods(vec![Method::GET, Method::POST, Method::PATCH])
                .allow_headers(vec![AUTHORIZATION, ACCEPT, CONTENT_TYPE])
                .allow_credentials(true),
        )
}
