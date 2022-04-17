#[macro_use]
mod custom_macro;
mod constants;
mod error;
mod extractor;
mod model;
mod response;
mod route;
mod storage;
mod util;

#[macro_use]
extern crate lazy_static;
use std::net::SocketAddr;
use tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::constants::ENVS;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&ENVS.rust_log))
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
