use std::ops::Sub;

use serde::Deserialize;

fn deserialize_unix_timestamp<'de, D>(
    deserializer: D,
) -> Result<chrono::DateTime<chrono::Local>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(
        chrono::DateTime::<chrono::Utc>::from_timestamp(i64::deserialize(deserializer)?, 0)
            .unwrap()
            .with_timezone(&chrono::Local),
    )
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Job {
    #[serde(
        alias = "starttime",
        deserialize_with = "deserialize_unix_timestamp",
        skip_serializing
    )]
    start: chrono::DateTime<chrono::Local>,
}

pub enum Status {
    Bad,
    Ok,
}

impl serde::Serialize for Status {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let inner = match self {
            Self::Bad => "BAD",
            Self::Ok => "OK",
        };
        serde_json::json!({"status": inner}).serialize(serializer)
    }
}

async fn latest_backup_job() -> axum::Json<Status> {
    let output = std::process::Command::new("pvesh")
        .args([
            "get",
            "/nodes/pve/tasks",
            "--typefilter",
            "vzdump",
            "--output-format",
            "json",
        ])
        .output()
        .unwrap();
    let job = serde_json::from_slice::<Vec<Job>>(&output.stdout)
        .unwrap()
        .swap_remove(0);

    axum::Json(
        if chrono::Local::now().sub(job.start).abs().num_days() > 1 {
            Status::Bad
        } else {
            Status::Ok
        },
    )
}

#[tokio::main]
async fn main() {
    let application = axum::Router::new().route("/backups", axum::routing::get(latest_backup_job));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to the correct port");
    axum::serve(listener, application)
        .await
        .expect("Failed to serve application");
}
