# hello-axum deploy

Ansible playbook to deploy the `hello-axum` Rust webhook server behind Caddy on a Debian/Ubuntu VPS.

## Prerequisites

- Ansible >= 2.14 on your workstation
- SSH access as `root` (or a sudo user) to `webhooks.jareddyreson.com`
- DNS A record pointing `webhooks.jareddyreson.com` at the VPS
- Built release binary at `target/release/hello-axum` (run `cargo build --release`)

## Install collections

```bash
ansible-galaxy collection install -r requirements.yaml
```

## Run

```bash
cargo build --release
ansible-playbook -i playbooks/inventory.yaml playbooks/deploy.yaml
```

Dry run:

```bash
ansible-playbook -i playbooks/inventory.yaml playbooks/deploy.yaml --check --diff
```

## What it does

1. Installs Caddy from the official apt repository
2. Creates the `hello-axum` system user and `/opt/hello-axum/`
3. Copies the release binary
4. Installs and starts the systemd unit from `deploy/hello-axum.service`
5. Installs the Caddyfile from `deploy/Caddyfile` and reloads Caddy
6. Opens ports 80 and 443 in ufw

Re-running the playbook after `cargo build --release` will push a new binary and restart the service.
