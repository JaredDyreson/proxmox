//! Pi-hole check.
//!
//! Confirms the `pihole-FTL` cron job ran today by inspecting the system journal.

/// Verify Pi-hole's `pihole-FTL` cron job has a journal entry for today.
pub fn check() -> anyhow::Result<()> {
    crate::backends::parse_cron("pihole-FTL")
}
