pub mod backends;
pub mod deserializers;
pub mod enums;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
}

/// Entry-point for all Uptime Kuma check endpoints
///
/// # Args
///
/// * `params`: URL-encoded instructions on what service we're interested in
async fn latest_backup_job(
    axum::extract::Query(params): axum::extract::Query<crate::backends::Query>,
) -> (axum::http::StatusCode, axum::Json<crate::enums::Status>) {
    let status = params.r#type.status();
    let code = match status {
        crate::enums::Status::Ok => axum::http::StatusCode::OK,
        crate::enums::Status::Bad { .. } => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    };
    (code, axum::Json(status))
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    logtail::setup_logger(log::LevelFilter::Info, &[]).expect("Failed to initialize the logger");
    let application = axum::Router::new().route("/backups", axum::routing::get(latest_backup_job));
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", cli.port))
        .await
        .expect("Failed to bind to the correct port");
    axum::serve(listener, application)
        .await
        .expect("Failed to serve application");
}
