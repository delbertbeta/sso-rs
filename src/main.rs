mod db;
mod route;

use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "sso_rs=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = route::get_app().await;
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
