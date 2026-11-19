#!/usr/bin/env bash
set -euo pipefail
cd "$HOME/uor-codex/20261118-content-support/panels"
test -s results/old/seed1-held.json && test -s results/role/seed1-held.json
jq -s -f negative.jq results/{role,old}/seed1-held.json > results/prefix-corruption-fixture.json
if jq --arg arm old --slurpfile p results/prefix-corruption-fixture.json --argjson grade '[]' \
  -f picks.jq results/role/seed1-held.json >/dev/null; then exit 1; fi
