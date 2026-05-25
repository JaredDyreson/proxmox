use anyhow::Context;
use axum::Router;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::TcpListener;

pub mod endpoints;
pub mod ledger;
pub mod schemas;

pub const LOCAL: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 3_000);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logtail::setup_logger(log::LevelFilter::Info, &[])?;

    let app = Router::new()
        .route("/", axum::routing::post(crate::endpoints::ingest))
        .route("/health", axum::routing::get(crate::endpoints::health));

    let listener = TcpListener::bind(LOCAL)
        .await
        .context("Failed to instantiate web server")?;

    log::info!("Listening on http://{LOCAL}");

    axum::serve(listener, app)
        .await
        .context("Failed to serve the web server")
}
