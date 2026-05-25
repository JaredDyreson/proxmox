//! Back-end implementations for each service the checker monitors.
//!
//! Each submodule exposes a `check()` function that returns `Ok(())` when the
//! service's most recent backup ran successfully and an `Err` otherwise. The
//! [`crate::enums::Backend`] enum dispatches to the appropriate `check()` based
//! on the incoming request.

pub mod pbs;
pub mod pfsense;
pub mod pihole;

use anyhow::Context;

/// URL-encoded request parameters to instruct the endpoint
#[derive(serde::Deserialize)]
pub struct Query {
    /// What back-end to target
    pub r#type: crate::enums::Backend,
}

/// Verify that a cron job identified by `command` ran today.
///
/// Shells out to `journalctl -t CRON --since today -g <command>` and parses the
/// first matching line's ISO-8601 timestamp. Returns `Ok(())` when a run is
/// found and its timestamp parses; returns `Err` if the command failed, no
/// matching log line exists, or the timestamp is malformed.
pub fn parse_cron(command: &str) -> anyhow::Result<()> {
    let output = std::process::Command::new("journalctl")
        .args([
            "-t",
            "CRON",
            "--since",
            "today",
            "-g",
            command,
            "-o",
            "short-iso",
        ])
        .output()
        .context("Failed to get `journalctl` output")?;
    output
        .status
        .success()
        .then_some(0)
        .context("Command failed")?;

    let buffer =
        String::from_utf8(output.stdout).context("Failed to convert standard out into a string")?;
    let timestamp = buffer
        .split_whitespace()
        .next()
        .context("Failed to get the timestamp from cron output")?;
    chrono::DateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%z")
        .context("Failed to parse the datetime string")?;
    Ok(())
}
