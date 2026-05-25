use anyhow::Context;

use std::net::{Ipv4Addr, SocketAddrV4};

pub const UPTIME_KUMA: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 120), 3_001);

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(untagged)]
pub enum WebhookMessage {
    GitHub(crate::schemas::github::StatusUpdate),
    Anthropic { name: String },
}

impl WebhookMessage {
    fn endpoint(&self) -> &'static str {
        match &self {
            Self::Anthropic { .. } => "bpGTDTavMpcJEisPbEZHCaurG6npYk89",
            Self::GitHub { .. } => "iyelFv2JY1BvGdCXCxVxFU0K3WaTuL7X",
        }
    }

    async fn notify_uptime(&self) -> anyhow::Result<()> {
        reqwest::Client::new()
            .post(format!(
                "http://{UPTIME_KUMA}/api/push/{}?status=up&msg=OK&ping=",
                self.endpoint()
            ))
            .json(&self)
            .send()
            .await
            .context("Failed to notify Uptime Kuma")?;

        Ok(())
    }
}

pub async fn ingest(axum::extract::Json(payload): axum::extract::Json<serde_json::Value>) {
    match serde_json::from_value::<WebhookMessage>(payload.clone()) {
        Ok(message) => message.notify_uptime().await.unwrap(),
        Err(_) => {
            log::error!("Invalid webhook message was received, placing in SQLite database");
            crate::ledger::Ledger::new(crate::ledger::URL.to_string())
                .await
                .unwrap()
                .insert_json(&payload)
                .await
                .unwrap();
        }
    }
}

pub async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(
        match crate::ledger::Ledger::new(crate::ledger::URL.to_string()).await {
            Ok(_) => serde_json::json!({"status": "OK"}),
            Err(_) => serde_json::json!({"status": "Ledger cannot be accessed"}),
        },
    )
}
