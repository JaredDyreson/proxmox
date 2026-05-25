//! Proxmox Backup Server (PBS) check.
//!
//! Queries the local Proxmox node via `pvesh` for recent `vzdump` tasks and
//! confirms the most recent one finished within the last day.

use std::ops::Sub;

use anyhow::Context;

/// A single `vzdump` task entry as returned by `pvesh`.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct Job {
    /// Wall-clock start time of the task, decoded from PBS's `starttime` unix epoch field.
    #[serde(
        alias = "starttime",
        deserialize_with = "crate::deserializers::deserialize_unix_timestamp",
        skip_serializing
    )]
    start: chrono::DateTime<chrono::Local>,
}

/// Verify that a `vzdump` backup task ran on the local node within the last 24 hours.
///
/// Invokes `pvesh get /nodes/pve/tasks --typefilter vzdump` and inspects the
/// newest entry. Returns `Err` if the command fails, output is empty or
/// malformed, or the most recent task is older than one day.
pub fn check() -> anyhow::Result<()> {
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
        .context("Failed to get `pvesh` output")?;
    output
        .status
        .success()
        .then_some(0)
        .context("Command failed")?;

    let job = serde_json::from_slice::<std::collections::VecDeque<Job>>(&output.stdout)
        .context("Failed to deserialize `pvesh` output")?
        .pop_front()
        .context("Failed to get first job")?;

    log::info!("Job: '{job:#?}'");

    (chrono::Local::now().sub(job.start).abs().num_days() <= 1)
        .then_some(0)
        .context("Last job did not run successfully")?;

    Ok(())
}
