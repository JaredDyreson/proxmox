//! Shared logging setup for workspace crates.

use anyhow::Context;

/// Install a `fern` logger that writes RFC 3339 timestamped records to stdout.
///
/// `level` is the blanket level filter; `module_levels` lets callers override
/// specific log targets (e.g. silencing noisy crates like `hyper` when running
/// at `Debug`).
pub fn setup_logger(
    level: log::LevelFilter,
    module_levels: &[(&str, log::LevelFilter)],
) -> anyhow::Result<()> {
    let mut dispatch = fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            let now = chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false);
            let (level, target) = (record.level(), record.target());
            out.finish(format_args!("[{level}][{now}][{target}] {message}",))
        })
        .level(level);

    for (module, module_level) in module_levels {
        dispatch = dispatch.level_for((*module).to_string(), *module_level);
    }

    dispatch
        .chain(std::io::stdout())
        .apply()
        .context("failed to apply changes")
}
