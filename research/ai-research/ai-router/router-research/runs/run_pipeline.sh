#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

mkdir -p results/raw results/parsed docs/governance/gates

python tools/sweep.py --config configs/route_sweep.yaml --log_dir results/raw --gate_dir docs/governance/gates
python tools/parse_logs.py results/raw results/parsed
python tools/summarize.py results/parsed results/summary.csv

echo "Pipeline complete. See results/summary.csv and docs/governance/gates/."
