#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
checker="$root/nanda-structural-gate/scripts/nanda-check"

set +e
"$checker" --triads "$root/examples/triad-packet.evidence-conflict.json" --format json >/tmp/nanda-edge-evidence.json
status=$?
set -e
if [[ "$status" -ne 1 ]]; then
  echo "expected evidence conflict to VETO" >&2
  cat /tmp/nanda-edge-evidence.json >&2
  exit 1
fi

set +e
"$checker" --triads "$root/examples/triad-packet.watch-low-complexity.json" --format json >/tmp/nanda-edge-watch.json
status=$?
set -e
if [[ "$status" -ne 3 ]]; then
  echo "expected low complexity packet to WATCH" >&2
  cat /tmp/nanda-edge-watch.json >&2
  exit 1
fi

echo "ok"
