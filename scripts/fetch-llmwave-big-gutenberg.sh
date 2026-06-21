#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out_dir="${1:-$root/.nanda/external-corpus/gutenberg}"
mkdir -p "$out_dir"

ids=(
  11
  84
  98
  345
  1260
  1342
  1400
  1661
  2701
  4300
)

for id in "${ids[@]}"; do
  url="https://www.gutenberg.org/cache/epub/$id/pg$id.txt"
  out="$out_dir/pg$id.txt"
  if [[ -s "$out" ]]; then
    continue
  fi
  wget --no-check-certificate -T 90 -t 2 -O "$out" "$url" || {
    rm -f "$out"
    echo "failed to fetch $url" >&2
    exit 1
  }
done

wc -l -w -c "$out_dir"/*.txt
