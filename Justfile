default:
    @just --list

# Run the Cucumber/Gherkin behavior suite.
bdd:
    cargo test --test bdd

# --- #273 template adoption: the register conformance gate ---
# R1/R2/R3 landed in Phase 1. #510 wires R4 (deferral) and R6 (cargo-deny):
# R4's `audit-deferral` now honours the carve-out --- a marker is a violation
# only when its line does not cite an `open` claim in model/ledger.toml --- and
# R6's deny.toml allow-lists uor-r4's pinned git sources. R5 (`audit-limits`) is
# still an incremental migration (352 unsanctioned `Result` returns as of #510)
# and is NOT yet in the gate.

# R1: model/*.toml is the single source; CONFORMANCE.md is generated from it.
check-model:
    cargo run -q -p xtask -- check-model

# Regenerate CONFORMANCE.md from the register after changing model/*.toml.
model-write:
    cargo run -q -p xtask -- check-model --write

# R2/R3: the honesty meta-gate and the ID-tagged BDD conformance suite.
register-bdd:
    cargo test -q -p repo-conformance

# R4: nothing is deferred and unregistered. A marker (TODO, unimplemented!, ...)
# fails unless its line cites an `open` ledger claim (the dormant carve-out).
audit-deferral:
    cargo run -q -p xtask -- audit-deferral

# R6: supply-chain hygiene over the real graph --- advisories, licences, bans,
# and sources (the pinned git deps are allow-listed in deny.toml).
deny:
    cargo deny check

# The register acceptance gate --- the wired slice of the template's `just vv`
# (R1/R2/R3 + R4 + R6). R5 joins when the audit-limits migration completes.
vv-register: check-model register-bdd audit-deferral deny
    @echo "vv-register: the register conformance gate passed (R1/R2/R3/R4/R6)"
