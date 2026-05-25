//! pfSense check.
//!
//! Confirms today's pfSense config backup landed by checking that the `scp`
//! cron job (which pulls the config off the firewall) ran today.

/// Verify the pfSense `scp` backup cron job has a journal entry for today.
pub fn check() -> anyhow::Result<()> {
    crate::backends::parse_cron("scp")
}
