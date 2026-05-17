#!/usr/bin/env bash
source <(curl -fsSL https://raw.githubusercontent.com/community-scripts/ProxmoxVE/main/misc/build.func)
# Copyright (c) 2021-2026 community-scripts ORG
# Author: jared <jared.dyreson@gmail.com>
# License: MIT | https://github.com/community-scripts/ProxmoxVE/raw/main/LICENSE
# Source: gitea@192.168.1.104:proxmox/kuma-filter.git

APP="kuma-filter"
var_tags="${var_tags:-monitoring;webhook}"
var_cpu="${var_cpu:-1}"
var_ram="${var_ram:-512}"
var_disk="${var_disk:-4}"
var_os="${var_os:-debian}"
var_version="${var_version:-12}"
var_unprivileged="${var_unprivileged:-1}"

header_info "$APP"
variables
color
catch_errors

SRC_DIR="${SRC_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

: "${PUSHOVER_TOKEN:?PUSHOVER_TOKEN must be set in the host environment}"
: "${PUSHOVER_USER:?PUSHOVER_USER must be set in the host environment}"

function update_script() {
  header_info
  check_container_storage
  check_container_resources

  if [[ ! -d /opt/kuma-filter ]]; then
    msg_error "No ${APP} Installation Found!"
    exit
  fi

  msg_info "Rebuilding ${APP}"
  systemctl stop kuma-filter
  source "$HOME/.cargo/env"
  cd /opt/kuma-filter
  cargo build --release
  systemctl start kuma-filter
  msg_ok "Updated ${APP}"
  exit
}

start
build_container
description

[[ -f "${SRC_DIR}/Cargo.toml" ]] || { msg_error "Cargo.toml not found in ${SRC_DIR} — set SRC_DIR to the cloned repo"; exit 1; }

msg_info "Installing build dependencies"
pct exec "$CTID" -- bash -c '
  set -e
  export DEBIAN_FRONTEND=noninteractive
  apt-get update -qq
  apt-get install -y -qq curl ca-certificates build-essential pkg-config libssl-dev
'
msg_ok "Installed build dependencies"

msg_info "Installing Rust toolchain"
pct exec "$CTID" -- bash -c '
  set -euo pipefail
  if command -v cargo >/dev/null 2>&1 || [[ -x /root/.cargo/bin/cargo ]]; then
    exit 0
  fi
  curl --retry 5 --retry-delay 3 --retry-connrefused -fsSL https://sh.rustup.rs -o /tmp/rustup.sh
  sh /tmp/rustup.sh -y --default-toolchain stable --profile minimal --no-modify-path
  rm -f /tmp/rustup.sh
  test -x /root/.cargo/bin/cargo
'
msg_ok "Installed Rust toolchain"

msg_info "Pushing source from ${SRC_DIR}"
TARBALL="$(mktemp --suffix=.tar.gz)"
trap 'rm -f "$TARBALL"' EXIT
tar -czf "$TARBALL" -C "$SRC_DIR" Cargo.toml Cargo.lock src
pct exec "$CTID" -- mkdir -p /opt/kuma-filter
pct push "$CTID" "$TARBALL" /opt/kuma-filter/src.tar.gz
pct exec "$CTID" -- tar -xzf /opt/kuma-filter/src.tar.gz -C /opt/kuma-filter
pct exec "$CTID" -- rm -f /opt/kuma-filter/src.tar.gz
msg_ok "Pushed source"

msg_info "Building release binary (this can take a few minutes)"
pct exec "$CTID" -- bash -lc 'source $HOME/.cargo/env && cd /opt/kuma-filter && cargo build --release'
msg_ok "Built release binary"

msg_info "Configuring kuma-filter"
ENVFILE="$(mktemp)"
chmod 600 "$ENVFILE"
trap 'rm -f "$TARBALL" "$ENVFILE"' EXIT
cat >"$ENVFILE" <<EOF
PUSHOVER_TOKEN=${PUSHOVER_TOKEN}
PUSHOVER_USER=${PUSHOVER_USER}
EOF
pct exec "$CTID" -- mkdir -p /etc/kuma-filter
pct push "$CTID" "$ENVFILE" /etc/kuma-filter/kuma-filter.env --perms 0600
pct exec "$CTID" -- bash -c "
  set -e
  cat >/etc/systemd/system/kuma-filter.service <<'EOF'
[Unit]
Description=kuma-filter Axum webhook for Uptime Kuma -> Pushover
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
EnvironmentFile=/etc/kuma-filter/kuma-filter.env
WorkingDirectory=/opt/kuma-filter
ExecStart=/opt/kuma-filter/target/release/kuma-filter
Restart=on-failure
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF
  systemctl daemon-reload
  systemctl enable --now kuma-filter
"
msg_ok "Configured kuma-filter"

msg_info "Writing LXC notes"
pct set "$CTID" --description "# kuma-filter

Axum webhook receiver that forwards Uptime Kuma notifications to Pushover.

## Endpoints
- POST \`http://${IP}:3000/\` — webhook target for Uptime Kuma

## Layout
- Binary: \`/opt/kuma-filter/target/release/kuma-filter\`
- Source: \`/opt/kuma-filter\` (pushed from PVE host, not a git clone)
- Service: \`kuma-filter.service\`
- Config:  \`/etc/kuma-filter/kuma-filter.env\` (\`PUSHOVER_TOKEN\`, \`PUSHOVER_USER\`)

## Operations
- Status:  \`systemctl status kuma-filter\`
- Logs:    \`journalctl -u kuma-filter -f\`
- Restart: \`systemctl restart kuma-filter\` (after editing the env file or rotating creds)

## Updating
Run from the cloned repo on the **Proxmox host** (not inside the LXC):
\`\`\`
bash ct/update-kuma-filter.sh ${CTID}
\`\`\`
This re-pushes source, rebuilds, and restarts the service.

## Upstream
\`gitea@192.168.1.104:proxmox/kuma-filter.git\`"
msg_ok "Wrote LXC notes"

msg_ok "Completed successfully!\n"
echo -e "${CREATING}${GN}${APP} setup has been successfully initialized!${CL}"
echo -e "${INFO}${YW} Pushover credentials written to /etc/kuma-filter/kuma-filter.env inside the LXC.${CL}"
echo -e "${INFO}${YW} Webhook endpoint:${CL}"
echo -e "${TAB}${GATEWAY}${BGN}http://${IP}:3000/${CL}"
