#[macro_use]
mod custom_macro;
mod db;
mod error;
mod response;
mod route;
mod util;

use std::net::SocketAddr;
use tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "sso_rs=debug,tower_http=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = route::get_app().await;
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
