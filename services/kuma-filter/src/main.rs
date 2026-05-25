//! Web application hosted in Proxmox to improve the message body Uptime Kuma
//! ships with by default. Emojis suck and the formatting is...interesting.

use anyhow::Context;
use axum::{Json, Router, routing::post};
use log::LevelFilter;

#[derive(serde::Deserialize)]
pub struct UptimeMessage {
    /// Name of the service
    #[serde(rename = "name")]
    service: String,
    /// How the  service is doing
    status: String,
    /// Contents of the message
    message: String,
}

/// Sends a Pushover notification when a service goes offline
///
/// The generic webhook integration allows you to define the format using `LucidJs`,
/// which allows you to place variables the JSON format string.
async fn notify_admin(
    Json(UptimeMessage {
        service,
        status,
        message,
    }): Json<UptimeMessage>,
) -> Json<String> {
    match message.contains("children") {
        true => {
            log::warn!("Ignoring this message...");
        }
        false => {
            let status = if status.contains("Down") {
                "down"
            } else {
                "up"
            };

            let form = reqwest::multipart::Form::new()
                .text("token", std::env!("PUSHOVER_TOKEN"))
                .text("user", std::env!("PUSHOVER_USER"))
                .text("message", format!("'{service}' is now {status}"));

            match reqwest::Client::new()
                .post("https://api.pushover.net/1/messages.json")
                .multipart(form)
                .send()
                .await
            {
                Ok(_) => {
                    log::info!("Successfully sent the message to Pushover")
                }
                Err(error) => {
                    log::error!("Failed to send the message to Pushover: '{error}'")
                }
            }
        }
    };

    Json(String::default())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logtail::setup_logger(
        LevelFilter::Debug,
        &[("hyper", LevelFilter::Info), ("h2", LevelFilter::Info)],
    )?;

    let app = Router::new().route("/", post(notify_admin));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    log::debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .await
        .context("Failed to serve the web server")
}
