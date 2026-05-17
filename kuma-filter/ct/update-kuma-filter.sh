#!/usr/bin/env bash
# Update a running kuma-filter LXC with fresh source from the local repo.
# Run on the Proxmox host from the cloned kuma-filter repo.
#
# Usage:
#   bash ct/update-kuma-filter.sh <CTID>
#   CTID=200 bash ct/update-kuma-filter.sh
#   SRC_DIR=/path/to/repo bash ct/update-kuma-filter.sh 200

set -euo pipefail

CTID="${1:-${CTID:-}}"
SRC_DIR="${SRC_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

[[ -n "$CTID" ]]                  || { echo "usage: $0 <CTID>" >&2; exit 2; }
command -v pct >/dev/null         || { echo "pct not found — run on the Proxmox host" >&2; exit 1; }
[[ -f "${SRC_DIR}/Cargo.toml" ]]  || { echo "Cargo.toml not found in ${SRC_DIR}" >&2; exit 1; }
pct status "$CTID" >/dev/null 2>&1 || { echo "LXC ${CTID} does not exist" >&2; exit 1; }

if [[ "$(pct status "$CTID")" != *running* ]]; then
  echo "Starting LXC ${CTID}..."
  pct start "$CTID"
fi

echo "Stopping kuma-filter service..."
pct exec "$CTID" -- systemctl stop kuma-filter || true

echo "Pushing source from ${SRC_DIR}..."
TARBALL="$(mktemp --suffix=.tar.gz)"
trap 'rm -f "$TARBALL"' EXIT
tar -czf "$TARBALL" -C "$SRC_DIR" Cargo.toml Cargo.lock src
pct exec "$CTID" -- mkdir -p /opt/kuma-filter
pct push "$CTID" "$TARBALL" /opt/kuma-filter/src.tar.gz
pct exec "$CTID" -- tar -xzf /opt/kuma-filter/src.tar.gz -C /opt/kuma-filter
pct exec "$CTID" -- rm -f /opt/kuma-filter/src.tar.gz

echo "Building release binary..."
pct exec "$CTID" -- bash -lc 'source $HOME/.cargo/env && cd /opt/kuma-filter && cargo build --release'

echo "Restarting service..."
pct exec "$CTID" -- systemctl start kuma-filter
pct exec "$CTID" -- systemctl --no-pager --lines=5 status kuma-filter || true

echo
echo "Updated kuma-filter in LXC ${CTID}."
