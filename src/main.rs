#[macro_use]
mod custom_macro;
mod constants;
mod error;
mod extractor;
mod model;
mod response;
mod route;
mod rpc;
mod storage;
mod util;

#[macro_use]
extern crate lazy_static;
use crate::constants::ENVS;
use crate::storage::{mysql, session};
use tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rpc::user::S;
use std::net::SocketAddr;
use volo_grpc::server::{Server, ServiceBuilder};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&ENVS.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let conn = mysql::get_mysql_db_conn().await;
    let session_store = session::get_session_store();

    let app = route::get_app(conn.clone(), session_store.clone()).await;
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let rpc_addr: SocketAddr = "[::]:2999".parse().unwrap();
    let rpc_addr = volo::net::Address::from(rpc_addr);

    tracing::debug!("\nlistening on \nHTTP: {}\nRPC: {}", addr, rpc_addr);

    let rpc_server = Server::new()
        .add_service(
            ServiceBuilder::new(volo_gen::sso::rs::UserServiceServer::new(S::new(
                conn.clone(),
                session_store.clone(),
            )))
            .build(),
        )
        .run(rpc_addr);

    let http_server = axum::serve(listener, app);

    tokio::select! {
        rpc = rpc_server => {
            rpc.unwrap();
        }
        http = http_server => {
            http.unwrap();
        }
    }
}
