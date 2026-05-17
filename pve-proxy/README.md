# pve-proxy

Axum HTTP server that exposes a small JSON status surface over the [Proxmox VE](https://www.proxmox.com/) API, intended to be polled by an external uptime monitor such as [Uptime Kuma](https://github.com/louislam/uptime-kuma).

Currently exposes one endpoint:

| Method | Path | Behavior |
|--------|------|----------|
| `GET`  | `/backups` | Shells out to `pvesh get /nodes/pve/tasks --typefilter vzdump` and returns `{"status": "OK"}` if the most recent vzdump task started within the last day, otherwise `{"status": "BAD"}`. |

The node name (`pve`) is hardcoded in `src/main.rs`.

## Running locally

```sh
cargo run --release
```

The server listens on `0.0.0.0:3000`. It must run somewhere `pvesh` is on `PATH` and authenticated against the cluster, which in practice means directly on a Proxmox host.

## Configuration

None. No environment variables, no config file.
