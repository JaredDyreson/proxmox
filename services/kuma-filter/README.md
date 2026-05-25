# kuma-filter

Axum webhook receiver that filters [Uptime Kuma](https://github.com/louislam/uptime-kuma) notifications and forwards them to [Pushover](https://pushover.net/). Replaces Kuma's default emoji-laden message body with a terse `'<service>' is now up|down`, and drops messages mentioning `children` (noisy parent/child status rollups).

## Configuration

Two environment variables are required and read at compile time via `std::env!`:

| Variable | Purpose |
|----------|---------|
| `PUSHOVER_TOKEN` | Pushover application token |
| `PUSHOVER_USER`  | Pushover user/group key |

Both must be set in the build environment; `cargo build` will fail if either is missing.

## Running locally

```sh
PUSHOVER_TOKEN=... PUSHOVER_USER=... cargo run --release
```

The server listens on `0.0.0.0:3000` and accepts `POST /` with a JSON body shaped like:

```json
{ "name": "service-name", "status": "Down", "message": "..." }
```

Point Uptime Kuma's generic webhook integration at it with that body template.

## Deploying to Proxmox

The `ct/` directory holds helper scripts modeled on the [community-scripts/ProxmoxVE](https://github.com/community-scripts/ProxmoxVE) format. Both scripts run on the **Proxmox host**, not inside the LXC.

- `ct/kuma-filter.sh`: provisions a new Debian 12 LXC, installs Rust, builds the release binary, writes `/etc/kuma-filter/kuma-filter.env`, and registers a `kuma-filter.service` systemd unit.
- `ct/update-kuma-filter.sh <CTID>`: re-pushes the current source tree into an existing LXC, rebuilds, and restarts the service.

Both scripts pick up `PUSHOVER_TOKEN` / `PUSHOVER_USER` from the host's environment.

## Layout inside the LXC

| Path | Contents |
|------|----------|
| `/opt/kuma-filter` | Source tree pushed from the host (not a git clone) |
| `/opt/kuma-filter/target/release/kuma-filter` | Release binary |
| `/etc/kuma-filter/kuma-filter.env` | Pushover credentials (0600) |
| `kuma-filter.service` | systemd unit |

Operate with the usual `systemctl status kuma-filter` / `journalctl -u kuma-filter -f`.
